use bevy::{
    camera::visibility::{NoFrustumCulling, RenderLayers},
    prelude::*,
    window::{PresentMode, Window, WindowPlugin},
};

#[cfg(target_arch = "wasm32")]
use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy_psx::{camera::PsxCamera, material::PsxMaterial, PsxPlugin};
use bevy_world_space_ui::WorldSpaceUiPlugin;

use bevy::{
    color::{palettes::css::BLACK, LinearRgba},
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImagePlugin, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::{vec4, Vec4},
    pbr::{ExtendedMaterial, MaterialExtension, MeshMaterial3d},
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    ui::ComputedNode,
};

const UI_LAYER: usize = 2;
const BUBBLE_WORLD_OFFSET: Vec3 = Vec3::new(0.0, 2.8, 0.0);
const BUBBLE_SCREEN_GAP: f32 = 10.0;

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
        .add_plugins(WorldSpaceUiPlugin)
        .add_plugins(PsxPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_mouse_look,
                update_speech_bubble_position.after(bevy_psx::camera::scale_render_image),
            ),
        )
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
        PlayerCamera,
        MouseLook::default(),
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
    commands.spawn((
        SceneRoot(asset_server.load("tnua_demon.glb#Scene0")),
        Transform::IDENTITY,
        Demon,
    ));
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

    let inter_font = asset_server.load("fonts/Inter-VariableFont_opsz,wght.ttf");

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.11, 0.43, 0.96, 0.95)),
            BorderRadius::all(Val::Px(40.0)),
            ZIndex(1),
            RenderLayers::layer(UI_LAYER),
            SpeechBubble {
                world_offset: BUBBLE_WORLD_OFFSET,
                screen_gap: BUBBLE_SCREEN_GAP,
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("h͛̿ͥͯëͭ̀̚l̄ͯ͋l͂͒ͪo̎͋̀ world! HAHAHHAHHA OMGOMGOMG kappa epic trollage"),
                TextFont {
                    font: inter_font,
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                RenderLayers::layer(UI_LAYER),
                Node::default(),
            ));
        });
}

#[derive(Component)]
struct Rotates;

#[derive(Component)]
struct Demon;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct MouseLook {
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    initialized: bool,
}

impl Default for MouseLook {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.009,
            initialized: false,
        }
    }
}

#[derive(Component)]
struct SpeechBubble {
    world_offset: Vec3,
    screen_gap: f32,
}

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

fn update_mouse_look(
    mut mouse_events: EventReader<bevy::input::mouse::MouseMotion>,
    mut query: Query<(&mut Transform, &mut MouseLook), With<PlayerCamera>>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_events.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO && query.is_empty() {
        return;
    }

    for (mut transform, mut look) in &mut query {
        if !look.initialized {
            transform.translation = Vec3::new(0.0, 2.0, 10.0);
            transform.look_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y);
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            look.yaw = yaw;
            look.pitch = pitch;
            look.initialized = true;
        }

        if delta != Vec2::ZERO {
            look.yaw -= delta.x * look.sensitivity;
            look.pitch = (look.pitch - delta.y * look.sensitivity).clamp(-1.54, 1.54);

            let yaw_rot = Quat::from_rotation_y(look.yaw);
            let pitch_rot = Quat::from_rotation_x(look.pitch);
            transform.rotation = yaw_rot * pitch_rot;
        }
    }
}

fn update_speech_bubble_position(
    mut bubble_query: Query<(&mut Node, &ComputedNode, &mut Visibility, &SpeechBubble)>,
    player_query: Query<&GlobalTransform, With<Demon>>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<PsxCamera>, With<PlayerCamera>)>,
    final_camera_query: Query<&Camera, With<bevy_psx::camera::FinalCameraTag>>,
) {
    let Ok((mut bubble_node, computed, mut visibility, bubble)) = bubble_query.single_mut() else {
        return;
    };
    let Ok(player_transform) = player_query.single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Ok(final_camera) = final_camera_query.single() else {
        *visibility = Visibility::Hidden;
        return;
    };

    let Ok(viewport_coords) = camera.world_to_viewport(
        camera_transform,
        player_transform.translation() + bubble.world_offset,
    ) else {
        *visibility = Visibility::Hidden;
        return;
    };

    let Some(base_viewport_size) = camera.logical_viewport_size() else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(final_viewport_size) = final_camera.logical_viewport_size() else {
        *visibility = Visibility::Hidden;
        return;
    };

    let viewport_origin = final_camera
        .viewport
        .as_ref()
        .and_then(|vp| final_camera.to_logical(vp.physical_position))
        .unwrap_or(Vec2::ZERO);

    let scale = Vec2::new(
        final_viewport_size.x / base_viewport_size.x.max(1e-6),
        final_viewport_size.y / base_viewport_size.y.max(1e-6),
    );

    if !scale.is_finite() {
        *visibility = Visibility::Hidden;
        return;
    }

    let bubble_size = computed.size();
    if bubble_size.x <= 0.0 || bubble_size.y <= 0.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    let screen_pos = viewport_origin + viewport_coords * scale;
    let bubble_left = screen_pos.x - bubble_size.x * 0.5;
    let bubble_top = screen_pos.y - bubble_size.y - bubble.screen_gap;

    bubble_node.left = Val::Px(bubble_left);
    bubble_node.top = Val::Px(bubble_top);
    *visibility = Visibility::Visible;
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
