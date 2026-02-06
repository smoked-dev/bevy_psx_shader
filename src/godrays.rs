use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    pbr::MeshMaterial3d,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Component)]
pub struct GodRaysTarget;

#[derive(Component)]
struct GodRaysSpawned;

#[derive(Component)]
struct GodRaysAnchor {
    target: Entity,
}

#[derive(Component)]
struct GodRay {
    velocity: Vec2,
    phase: f32,
    pulse_speed: f32,
    base_alpha: f32,
}

#[derive(Resource, Clone)]
pub struct GodRaysSettings {
    pub count: usize,
    pub area: Vec2,
    pub depth_range: Vec2,
    pub width_range: Vec2,
    pub length_range: Vec2,
    pub drift_dir: Vec2,
    pub drift_speed_range: Vec2,
    pub pulse_speed_range: Vec2,
    pub alpha_range: Vec2,
    pub pulse_power: f32,
    pub tint: Color,
    pub wrap_margin: f32,
}

impl Default for GodRaysSettings {
    fn default() -> Self {
        Self {
            count: 120,
            area: Vec2::new(80.0, 45.0),
            depth_range: Vec2::new(18.0, 80.0),
            width_range: Vec2::new(1.8, 3.8),
            length_range: Vec2::new(10.0, 28.0),
            drift_dir: Vec2::new(-1.0, -0.15),
            drift_speed_range: Vec2::new(1.0, 5.0),
            pulse_speed_range: Vec2::new(0.6, 1.8),
            alpha_range: Vec2::new(0.05, 0.35),
            pulse_power: 3.0,
            tint: Color::srgba(0.88, 0.97, 1., 1.0),
            wrap_margin: 8.0,
        }
    }
}

#[derive(Resource)]
struct GodRaysAssets {
    mesh: Handle<Mesh>,
    texture: Handle<Image>,
}

#[derive(Resource)]
struct GodRaysRng(u32);

impl GodRaysRng {
    fn seeded() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| (d.as_nanos() as u64) ^ ((d.as_secs() as u64) << 32))
            .unwrap_or(0x1234_5678_90ab_cdef);
        Self((seed as u32) ^ ((seed >> 32) as u32) ^ 0x9e37_79b9)
    }

    fn next_u32(&mut self) -> u32 {
        // xorshift32
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        // [0, 1)
        let v = self.next_u32() >> 8;
        (v as f32) / ((1u32 << 24) as f32)
    }

    fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    fn range_vec2(&mut self, min: Vec2, max: Vec2) -> Vec2 {
        Vec2::new(
            self.range_f32(min.x, max.x),
            self.range_f32(min.y, max.y),
        )
    }
}

pub struct GodRaysPlugin;

impl Plugin for GodRaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodRaysSettings>()
            .insert_resource(GodRaysRng::seeded())
            .add_systems(Startup, init_god_rays_assets)
            .add_systems(
                Update,
                (
                    spawn_god_rays_for_targets,
                    sync_god_rays_anchors,
                    cleanup_orphaned_god_rays_anchors,
                    drift_god_rays,
                    pulse_god_rays,
                ),
            );
    }
}

fn init_god_rays_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut images: ResMut<Assets<Image>>) {
    let mesh = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
    let texture = images.add(make_god_ray_texture(64, 256));
    commands.insert_resource(GodRaysAssets { mesh, texture });
}

fn make_god_ray_texture(width: u32, height: u32) -> Image {
    let mut data = vec![0u8; (width * height * 4) as usize];

    let smoothstep = |edge0: f32, edge1: f32, x: f32| -> f32 {
        if (edge1 - edge0).abs() < f32::EPSILON {
            return if x < edge0 { 0.0 } else { 1.0 };
        }
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };

    for y in 0..height {
        let yf = if height > 1 {
            y as f32 / (height - 1) as f32
        } else {
            0.0
        };
        // Cartoony ray: sharp sides (no left/right fade), only fades along the ray length (v).
        // Fade in quickly near v=0, then fade out near v=1.
        let fade_in = smoothstep(0.0, 0.04, yf);
        let fade_out = 1.0 - smoothstep(0.70, 1.0, yf);
        let length_fade = (fade_in * fade_out).clamp(0.0, 1.0);

        for x in 0..width {
            let alpha = length_fade;

            let i = ((y * width + x) * 4) as usize;
            data[i] = 255;
            data[i + 1] = 255;
            data[i + 2] = 255;
            data[i + 3] = (alpha * 255.0) as u8;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });

    image
}

fn spawn_god_rays_for_targets(
    mut commands: Commands,
    settings: Res<GodRaysSettings>,
    assets: Res<GodRaysAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rng: ResMut<GodRaysRng>,
    targets: Query<Entity, (With<GodRaysTarget>, With<Camera3d>, Without<GodRaysSpawned>)>,
) {
    let drift_base = if settings.drift_dir.length_squared() > 0.0001 {
        settings.drift_dir.normalize()
    } else {
        Vec2::new(-1.0, 0.0)
    };
    let half = settings.area * 0.5;

    for target in &targets {
        // `PsxCamera` inserts `Visibility::Hidden` on the camera entity, which would hide any
        // descendants. We instead spawn an external anchor and keep it synced to the camera's
        // transform, then parent all rays under that anchor.
        let anchor = commands
            .spawn((
                Name::new("GodRaysAnchor"),
                GodRaysAnchor { target },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        commands.entity(target).insert(GodRaysSpawned);

        commands.entity(anchor).with_children(|parent| {
            for i in 0..settings.count {
                let depth = rng.range_f32(settings.depth_range.x, settings.depth_range.y);
                let pos = rng.range_vec2(-half, half);
                let width = rng.range_f32(settings.width_range.x, settings.width_range.y);
                let length = rng.range_f32(settings.length_range.x, settings.length_range.y);
                let twist = rng.range_f32(0.0, std::f32::consts::TAU);

                let drift_speed = rng.range_f32(
                    settings.drift_speed_range.x,
                    settings.drift_speed_range.y,
                );
                let drift_jitter = rng.range_vec2(Vec2::splat(-0.35), Vec2::splat(0.35));
                let velocity = (drift_base + drift_jitter).normalize_or_zero() * drift_speed;

                let pulse_speed = rng.range_f32(
                    settings.pulse_speed_range.x,
                    settings.pulse_speed_range.y,
                );
                let phase = rng.range_f32(0.0, std::f32::consts::TAU);
                let base_alpha =
                    rng.range_f32(settings.alpha_range.x, settings.alpha_range.y).clamp(0.0, 1.0);

                let material = materials.add(StandardMaterial {
                    base_color: settings.tint.with_alpha(0.0),
                    base_color_texture: Some(assets.texture.clone()),
                    unlit: true,
                    alpha_mode: AlphaMode::Add,
                    cull_mode: None,
                    ..default()
                });

                parent.spawn((
                    Name::new(format!("GodRay({i})")),
                    GodRay {
                        velocity,
                        phase,
                        pulse_speed,
                        base_alpha,
                    },
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(pos.x, pos.y, -depth))
                        .with_rotation(Quat::from_rotation_z(twist))
                        .with_scale(Vec3::new(width, length, 1.0)),
                    NoFrustumCulling,
                ));
            }
        });
    }
}

fn sync_god_rays_anchors(
    targets: Query<&GlobalTransform>,
    mut anchors: Query<(&GodRaysAnchor, &mut Transform)>,
) {
    for (anchor, mut transform) in &mut anchors {
        if let Ok(target_global) = targets.get(anchor.target) {
            *transform = Transform {
                translation: target_global.translation(),
                rotation: target_global.rotation(),
                scale: target_global.scale(),
            };
        }
    }
}

fn cleanup_orphaned_god_rays_anchors(
    mut commands: Commands,
    targets: Query<(), With<GodRaysTarget>>,
    anchors: Query<(Entity, &GodRaysAnchor)>,
) {
    for (anchor_entity, anchor) in &anchors {
        if targets.get(anchor.target).is_err() {
            commands.entity(anchor_entity).despawn();
        }
    }
}

fn drift_god_rays(
    time: Res<Time>,
    settings: Res<GodRaysSettings>,
    mut rng: ResMut<GodRaysRng>,
    mut rays: Query<(&mut GodRay, &mut Transform)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let half = settings.area * 0.5;
    let margin = settings.wrap_margin.max(0.0);

    for (mut ray, mut transform) in &mut rays {
        transform.translation.x += ray.velocity.x * dt;
        transform.translation.y += ray.velocity.y * dt;

        let mut wrapped = false;
        if transform.translation.x > half.x + margin {
            transform.translation.x = -half.x - margin;
            wrapped = true;
        } else if transform.translation.x < -half.x - margin {
            transform.translation.x = half.x + margin;
            wrapped = true;
        }

        if transform.translation.y > half.y + margin {
            transform.translation.y = -half.y - margin;
            wrapped = true;
        } else if transform.translation.y < -half.y - margin {
            transform.translation.y = half.y + margin;
            wrapped = true;
        }

        if wrapped {
            ray.phase = rng.range_f32(0.0, std::f32::consts::TAU);
            ray.pulse_speed =
                rng.range_f32(settings.pulse_speed_range.x, settings.pulse_speed_range.y);
            ray.base_alpha = rng
                .range_f32(settings.alpha_range.x, settings.alpha_range.y)
                .clamp(0.0, 1.0);
            transform.translation.z = -rng.range_f32(settings.depth_range.x, settings.depth_range.y);
        }
    }
}

fn pulse_god_rays(
    time: Res<Time>,
    settings: Res<GodRaysSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rays: Query<(&GodRay, &MeshMaterial3d<StandardMaterial>)>,
) {
    let t = time.elapsed_secs();
    let pulse_power = settings.pulse_power.max(0.0);

    for (ray, mat_handle) in &rays {
        let phase = t * ray.pulse_speed + ray.phase;
        let mut pulse = phase.sin() * 0.5 + 0.5;
        if pulse_power > 0.0 {
            pulse = pulse.powf(pulse_power);
        }

        let alpha = (ray.base_alpha * pulse).clamp(0.0, 1.0);

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = settings.tint.with_alpha(alpha);
        }
    }
}
