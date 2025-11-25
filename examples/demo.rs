use bevy::{
    prelude::*,
    render::view::NoFrustumCulling,
    window::{PresentMode, Window, WindowPlugin},
};
use bevy_psx::{
    camera::PsxCamera,
    material::PsxMaterial,
    PsxPlugin,
};

use bevy::{
    asset::RenderAssetUsages,
    color::{palettes::css::BLACK, LinearRgba},
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::{vec4, Vec4},
    pbr::{ExtendedMaterial, MaterialExtension, MeshMaterial3d},
    render::{
        mesh::Mesh3d,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat},
        texture::ImagePlugin,
    },
    sprite::SpriteBundle,
    text::{Text2d, TextFont, TextLayout},
};

const BUBBLE_WORLD_OFFSET: Vec3 = Vec3::new(0.0, 2.8, 0.0);
const BUBBLE_TEXTURE_SIZE: Extent3d = Extent3d {
    width: 512,
    height: 256,
    depth_or_array_layers: 1,
};
fn main() {
    App::new()
    //    .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins({
            let mut plugins = DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some({
                        let mut window = Window {
                            present_mode: PresentMode::AutoVsync,
                            fit_canvas_to_parent: true,
                            ..default()
                        };
                        #[cfg(target_arch = "wasm32")]
                        {
                            window.canvas = Some("#bevy-canvas".into());
                            window.prevent_default_event_handling = false;
                        }
                        window
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest());
            #[cfg(target_arch = "wasm32")]
            {
                plugins = plugins.set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                });
            }
            plugins
        })
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        //    .add_plugins(DefaultPlugins)
        .add_plugins(PsxPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_psx_camera, face_bubble_to_camera))
        //    .add_systems(Update,rotate)
        //    .add_systems(Update,render_image_scale2.after(scale_render_image))
        .run();
}

// RN3 TEST LOLOL

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    //   _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<PsxMaterial>>,
    mut smaterials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        PsxCamera::new(
            UVec2::new(1920 / 2, 1080 / 2),
            None,
            Color::srgba(0., 0., 0., 1.),
            true,
            32.,
            45.,
            1,
        ),
    //    Bloom::default(),
        OrbitingCamera,
    ));
    let transform =
        Transform::from_scale(Vec3::splat(0.20)).with_translation(Vec3::new(0.0, -3.5, -10.0));
    /*     commands.spawn((
        MaterialMeshBundle {
            mesh: asset_server.load("dvaBlender.glb#Mesh2/Primitive0"),
            material: materials.add(PsxMaterial {
                color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
                snap_amount: 10.0,
                fog_distance: Vec2::new(250.0, 750.0),

                ..Default::default()
            }),
            transform,
            ..default()
        },
        Rotates,
    ));
    commands.spawn((
        MaterialMeshBundle {
            mesh: asset_server.load("dvaBlender.glb#Mesh0/Primitive0"),
            material: materials.add(PsxMaterial {
                color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
                snap_amount: 10.0,
                fog_distance: Vec2::new(250.0, 750.0),

                ..Default::default()
            }),
            transform,
            ..default()
        },
        Rotates,
    ));
    commands.spawn((
        MaterialMeshBundle {
            //import from gltf dvaBlender.glb
            mesh: asset_server.load("dvaBlender.glb#Mesh1/Primitive0"),
            material: materials.add(PsxMaterial {
               // color_texture load from gltf
               // color_texture: Some(asset_server.load("crate.png")),
                color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
                snap_amount: 10.0,
                fog_distance: Vec2::new(250.0, 750.0),

                ..Default::default()
            }),
            transform,
            ..default()
        },
        Rotates,
    )); */
/*     spawn_water(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut water_materials,
    );
 */
    commands.spawn((SceneRoot(asset_server.load("skybox3.glb#Scene0")), Transform::from_scale(Vec3::splat(1.)), NoFrustumCulling));
    let demon = commands
        .spawn((
            SceneRoot(asset_server.load("tnua_demon.glb#Scene0")),
            Transform::IDENTITY,
            Demon,
        ))
        .id();
/* 
    commands.spawn((
        Mesh3d(asset_server.load("dvaBlender.glb#Mesh2/Primitive0")),
        MeshMaterial3d(smaterials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
            perceptual_roughness: 0.,
            ..Default::default()
        })),
        transform,
        Rotates,
    )); */
/*     commands.spawn((
        Mesh3d(asset_server.load("dvaBlender.glb#Mesh0/Primitive0")),
        MeshMaterial3d(smaterials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
            perceptual_roughness: 0.,
            ..Default::default()
        })),
        transform,
        Rotates,
    ));
    commands.spawn((
        Mesh3d(asset_server.load("dvaBlender.glb#Mesh1/Primitive0")),
        MeshMaterial3d(smaterials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("dvaBlender.glb#Texture0")),
            perceptual_roughness: 0.,
            ..Default::default()
        })),
        transform,
        Rotates,
    )); */
    commands.spawn((
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
    ));

    // World-space bubble textured quad above the demon so it goes through the PSX pass.
    let inter_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let bubble_texture = make_bubble_texture(&mut images);
    let bubble_size = Vec2::new(1.5, 0.7);

    let surface = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    image: bubble_texture.clone(),
                    custom_size: Some(bubble_size),
                    ..default()
                },
                transform: Transform::from_translation(BUBBLE_WORLD_OFFSET),
                ..default()
            },
            BubbleSurface,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("hello world"),
                TextFont {
                    font: inter_font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.01)),
            ));
        })
        .id();

    commands.entity(demon).add_child(surface);
}

#[derive(Component)]
struct Rotates;

#[derive(Component)]
struct OrbitingCamera;

#[derive(Component)]
struct Demon;

#[derive(Component)]
struct BubbleSurface;

/// Rotates any entity around the x and y axis
#[allow(dead_code)]
fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in &mut query {
        transform.rotate_y(0.95 * time.delta_secs());
        // transform.scale = Vec3::splat(0.25);
        // transform.rotate_x(0.95 * time.delta_seconds());
        // transform.rotate_z(0.95 * time.delta_seconds());
    }
}

fn orbit_psx_camera(time: Res<Time>, mut query: Query<&mut Transform, With<OrbitingCamera>>) {
    let angle = time.elapsed_secs() * 0.3;
    let x_radius = 0.4;
    let y_radius = 0.25;
    for mut transform in &mut query {
        transform.translation = Vec3::new(angle.cos() * x_radius, angle.sin() * y_radius + 2., 10.);
    }
}

fn make_bubble_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let width = BUBBLE_TEXTURE_SIZE.width as usize;
    let height = BUBBLE_TEXTURE_SIZE.height as usize;
    let mut data = vec![0u8; width * height * 4];
    let radius = 40.0_f32;
    let r_sq = radius * radius;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let corner_x = if px < radius {
                radius
            } else if px > (width as f32 - radius) {
                width as f32 - radius
            } else {
                px
            };
            let corner_y = if py < radius {
                radius
            } else if py > (height as f32 - radius) {
                height as f32 - radius
            } else {
                py
            };

            let dx = px - corner_x;
            let dy = py - corner_y;
            let dist_sq = dx * dx + dy * dy;
            let inside = dist_sq <= r_sq;

            let (r, g, b, a) = if inside {
                // Facebook-ish blue with slight transparency.
                (23u8, 120u8, 242u8, 240u8)
            } else {
                (23u8, 120u8, 242u8, 0u8)
            };

            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = a;
        }
    }

    let mut image = Image::new(
        BUBBLE_TEXTURE_SIZE,
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    images.add(image)
}

fn face_bubble_to_camera(
    camera_query: Query<&GlobalTransform, With<PsxCamera>>,
    demon_query: Query<&GlobalTransform, With<Demon>>,
    mut surfaces: Query<&mut Transform, With<BubbleSurface>>,
) {
    let Ok(cam_transform) = camera_query.get_single() else {
        return;
    };
    let Ok(demon_transform) = demon_query.get_single() else {
        return;
    };

    let camera_pos = cam_transform.translation();
    let demon_pos = demon_transform.translation();

    for mut surface in &mut surfaces {
        let world_pos = demon_pos + surface.translation;
        let look_rot =
            Transform::from_translation(world_pos).looking_at(camera_pos, Vec3::Y).rotation;
        surface.rotation = look_rot;
    }
}

fn color_to_vec4(color: Color) -> Vec4 {
    let linear: LinearRgba = color.into();
    Vec4::new(linear.red, linear.green, linear.blue, linear.alpha)
}
/* pub fn render_image_scale2(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut pixel_meshes: Query<&Mesh2dHandle, With<RenderImage>>,
    mut pixel_cameras: Query<&mut PsxCamera>,
    mut cameras: Query<&mut Camera>,
    windows: Query<&Window>,
) {
    if time.elapsed_secs() < 2. {
        return;
    }

    for window in windows.iter() {


        for mut psx_camera in pixel_cameras.iter_mut() {
            for mut camera in cameras.iter_mut() {
                if let Some(image_handle) = camera.target.as_image() {
                    if let Some(image) = images.get_mut(image_handle) {
                        let window_size = UVec2::new(window.resolution.physical_width(), window.resolution.physical_height());



                        let size = Extent3d {
                            width: window_size.x / 2,
                            height: window_size.y / 2,
                            ..default()
                        };


                        if image.size() != UVec2::new(size.width, size.height) {
                            psx_camera.size = UVec2::new(size.width, size.height);
                            println!("FAG");
                            image.resize(size);
                            for pixel_mesh in pixel_meshes.iter() {
                                if let Some(mesh) = meshes.get_mut(pixel_mesh.0.clone()) {
                                    *mesh = Mesh::from(Rectangle::new(
                                        size.width as f32,
                                        size.height as f32,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

    }

}
 */

// Spawns the water plane.
fn spawn_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    water_materials: &mut Assets<ExtendedMaterial<StandardMaterial, Water>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: BLACK.into(),
                perceptual_roughness: 0.0,
                ..default()
            },
            extension: Water {
                normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                    "textures/water_normals.png",
                    |settings| {
                        settings.is_srgb = false;
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    },
                ),
                // These water settings are just random values to create some
                // variety.
                settings: WaterSettings {
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),
                        vec4(0.153, 0.138, -0.149, -0.195),
                    ],
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 5.0,
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044),
                },
            },
        })),
        Transform::from_scale(Vec3::splat(100.0)).with_translation(Vec3::Y * -1.),
    ));
}

/// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct Water {
    /// The normal map image.
    ///
    /// Note that, like all normal maps, this must not be loaded as sRGB.
    #[texture(100)]
    #[sampler(101)]
    normals: Handle<Image>,

    // Parameters to the water shader.
    #[uniform(102)]
    settings: WaterSettings,
}

/// Parameters to the water shader.
#[derive(ShaderType, Debug, Clone)]
struct WaterSettings {
    /// How much to displace each octave each frame, in the u and v directions.
    /// Two octaves are packed into each `vec4`.
    octave_vectors: [Vec4; 2],
    /// How wide the waves are in each octave.
    octave_scales: Vec4,
    /// How high the waves are in each octave.
    octave_strengths: Vec4,
}

impl MaterialExtension for Water {
    fn deferred_fragment_shader() -> ShaderRef {
        "water_material.wgsl".into()
    }
}
