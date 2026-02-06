use bevy::{
    asset::{AssetMetaCheck, AssetPlugin},
    camera::visibility::NoFrustumCulling,
    image::ImagePlugin,
    prelude::*,
    ui::{ComputedNode, UiTargetCamera},
    window::{PresentMode, Window, WindowPlugin},
};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_psx::{
    camera::{FinalCameraTag, PsxCamera, RenderImage, ResolutionScale},
    godrays::{GodRaysPlugin, GodRaysTarget},
    line_boil::{LineBoil, LineBoilMaterial, LineBoilPlugin},
    material::{PsxDitherMaterial, PsxMaterial},
    overlay::{OverlayPlugin, RenderOnTop},
    skybox_tint::{SkyboxTint, SkyboxTintPlugin},
    PsxPlugin,
};

use bevy::{
    color::{palettes::css::BLACK, LinearRgba},
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::{vec4, Vec4},
    pbr::{ExtendedMaterial, MaterialExtension, MeshMaterial3d},
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};
use std::collections::HashMap;

// ============================================================================
// Emote System
// ============================================================================

/// Stores information about a registered emote
#[derive(Clone)]
struct EmoteInfo {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frame_count: usize,
    frame_duration: f32,
}

/// Resource that holds all registered emotes
#[derive(Resource, Default)]
struct EmoteRegistry {
    emotes: HashMap<String, EmoteInfo>,
}

impl EmoteRegistry {
    fn register(
        &mut self,
        name: &str,
        image: Handle<Image>,
        layout: Handle<TextureAtlasLayout>,
        frame_count: usize,
        frame_duration: f32,
    ) {
        self.emotes.insert(
            name.to_string(),
            EmoteInfo {
                image,
                layout,
                frame_count,
                frame_duration,
            },
        );
    }

    fn get(&self, name: &str) -> Option<&EmoteInfo> {
        self.emotes.get(name)
    }
}

/// Component for animated emotes
#[derive(Component)]
struct AnimatedEmote {
    timer: Timer,
    frame_count: usize,
}

/// Parsed segment of rich text - either plain text or an emote
enum RichTextSegment {
    Text(String),
    Emote(String),
}

/// Parses text like "Hello :ravecat: world" into segments
fn parse_rich_text(input: &str) -> Vec<RichTextSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ':' {
            // Look for closing ':'
            let mut emote_name = String::new();
            let mut found_closing = false;

            for next_c in chars.by_ref() {
                if next_c == ':' {
                    found_closing = true;
                    break;
                }
                if next_c.is_whitespace() {
                    // Not a valid emote, restore
                    emote_name.insert(0, ':');
                    current_text.push_str(&emote_name);
                    current_text.push(next_c);
                    break;
                }
                emote_name.push(next_c);
            }

            if found_closing && !emote_name.is_empty() {
                // Push any accumulated text first
                if !current_text.is_empty() {
                    segments.push(RichTextSegment::Text(current_text.clone()));
                    current_text.clear();
                }
                segments.push(RichTextSegment::Emote(emote_name));
            } else if !found_closing {
                // Reached end without finding closing ':'
                current_text.push(':');
                current_text.push_str(&emote_name);
            }
        } else {
            current_text.push(c);
        }
    }

    // Push remaining text
    if !current_text.is_empty() {
        segments.push(RichTextSegment::Text(current_text));
    }

    segments
}

/// Spawns rich text with emotes as a flex container
fn spawn_rich_text(
    commands: &mut Commands,
    emote_registry: &EmoteRegistry,
    text: &str,
    font: Handle<Font>,
    font_size: f32,
    text_color: Color,
) -> Entity {
    let segments = parse_rich_text(text);
    let emote_size = font_size; // Match emote size to font size

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(2.0),
                row_gap: Val::Px(2.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0, 132, 255)),
            BorderRadius::all(Val::Px(32.0)),
        ))
        .with_children(|parent| {
            for segment in segments {
                match segment {
                    RichTextSegment::Text(txt) => {
                        parent.spawn((
                            Text::new(txt),
                            TextFont {
                                font: font.clone(),
                                font_size,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    }
                    RichTextSegment::Emote(name) => {
                        if let Some(emote_info) = emote_registry.get(&name) {
                            parent.spawn((
                                ImageNode::from_atlas_image(
                                    emote_info.image.clone(),
                                    TextureAtlas {
                                        layout: emote_info.layout.clone(),
                                        index: 0,
                                    },
                                ),
                                Node {
                                    width: Val::Px(emote_size),
                                    height: Val::Px(emote_size),
                                    ..default()
                                },
                                AnimatedEmote {
                                    timer: Timer::from_seconds(
                                        emote_info.frame_duration,
                                        TimerMode::Repeating,
                                    ),
                                    frame_count: emote_info.frame_count,
                                },
                            ));
                        } else {
                            // Unknown emote, render as text
                            parent.spawn((
                                Text::new(format!(":{}:", name)),
                                TextFont {
                                    font: font.clone(),
                                    font_size,
                                    ..default()
                                },
                                TextColor(text_color),
                            ));
                        }
                    }
                }
            }
        })
        .id()
}

/// System to animate emotes
fn animate_emotes(
    time: Res<Time>,
    mut query: Query<(&mut AnimatedEmote, &mut ImageNode)>,
) {
    for (mut emote, mut image_node) in &mut query {
        emote.timer.tick(time.delta());
        if emote.timer.just_finished() {
            if let Some(atlas) = &mut image_node.texture_atlas {
                atlas.index = (atlas.index + 1) % emote.frame_count;
            }
        }
    }
}

fn main() {
    App::new()
    //    .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .init_resource::<EmoteRegistry>()
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
        .add_plugins(EguiPlugin::default())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        //    .add_plugins(DefaultPlugins)
        .add_plugins(PsxPlugin)
        .add_plugins(OverlayPlugin)
        .add_plugins(GodRaysPlugin)
        .add_plugins(LineBoilPlugin)
        .add_plugins(SkyboxTintPlugin)
        .init_resource::<ChromaticAberrationUi>()
        .init_resource::<LineBoilUi>()
        .init_resource::<DepthBiasUi>()
        .init_resource::<SkyboxTintUi>()
        // Disable bevy_egui auto-attach so it doesn't attach to the PsxCamera
        .add_systems(PreStartup, disable_egui_auto_create)
        // Attach egui to FinalCameraTag so it renders on top of PSX output
        .add_systems(PostUpdate, attach_egui_to_final_camera)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                freecam_movement,
                sync_speech_bubble.after(freecam_movement),
                animate_emotes,
                tag_demon_eyes,
                apply_chromatic_aberration_to_material,
                apply_line_boil_ui,
                apply_depth_bias_ui,
                apply_skybox_tint_ui,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            (
                chromatic_aberration_ui,
                line_boil_ui,
                depth_bias_ui,
                skybox_tint_ui,
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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut emote_registry: ResMut<EmoteRegistry>,
) {
    // Register the ravecat emote (8x10 grid of 112x112 sprites = 80 frames)
    let ravecat_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(112),
        8,  // columns
        10, // rows
        None,
        None,
    ));
    emote_registry.register(
        "ravecat",
        asset_server.load("rave-party_spritesheet.png"),
        ravecat_layout,
        80, // 8 * 10 = 80 frames
        0.1, // 50ms per frame = 20 FPS animation
    );
    let psx_camera = commands.spawn((
        PsxCamera::new(
            UVec2::new(1920 / 2, 1080 / 2),
            None,
            Color::srgba(0., 0., 0., 1.),
            true,
            32.,
            45.,
            1,
        ),
        Transform::from_xyz(0.0, 2.0, 10.0),
    //    Bloom::default(),
        FreeCam::default(),
        Msaa::Off,
    ))
    .id();

  //  commands.entity(psx_camera).insert(GodRaysTarget);

    // Spawn the rich text bubble with emotes
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let rich_text_bubble = spawn_rich_text(
        &mut commands,
        &emote_registry,
        "Hello :ravecat: world :ravecat:",
        font,
        28.0,
        Color::WHITE,
    );

    // Add positioning and orbiting behavior to the bubble
    commands.entity(rich_text_bubble).insert((
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        SpeechBubble {
            world_offset: SPEECH_BUBBLE_WORLD_OFFSET,
            screen_offset: SPEECH_BUBBLE_SCREEN_OFFSET,
        },
        Name::new("SpeechBubble"),
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            SpeechBubbleRoot,
            Name::new("SpeechBubbleRoot"),
        ))
        .add_child(rich_text_bubble);

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
    commands.spawn((
        SceneRoot(asset_server.load("skybox3.glb#Scene0")),
        Transform::from_scale(Vec3::splat(1.)),
        NoFrustumCulling,
        SkyboxTint::new(),
        Skybox,
    ));
    // First demon (front)
 /*    commands.spawn((
        SceneRoot(asset_server.load("tnua_demon3.glb#Scene0")),
        Transform::IDENTITY,
        Demon,
        LineBoil::subtle(),
    ));*/
    // Second demon (behind, offset to the right)
    commands.spawn((
        SceneRoot(asset_server.load("tnua_demon5.glb#Scene0")),
        Transform::from_translation(Vec3::new(1.5, 0.0, -4.0)),
        Demon,
        LineBoil::subtle(),
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
}

/// Disable bevy_egui's automatic context creation so it doesn't attach to PsxCamera
fn disable_egui_auto_create(mut egui_settings: ResMut<EguiGlobalSettings>) {
    egui_settings.auto_create_primary_context = false;
}

/// Attach egui context to FinalCameraTag so egui renders on top of PSX output
fn attach_egui_to_final_camera(
    mut commands: Commands,
    final_camera_query: Query<Entity, (With<FinalCameraTag>, Without<EguiContext>)>,
) {
    for entity in &final_camera_query {
        commands.entity(entity).insert((
            EguiContext::default(),
            PrimaryEguiContext,
        ));
    }
}

#[derive(Component)]
struct Rotates;

#[derive(Component)]
struct OrbitingCamera;

#[derive(Component)]
struct FreeCam {
    speed: f32,
    sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for FreeCam {
    fn default() -> Self {
        Self {
            speed: 5.0,
            sensitivity: 0.003,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Component)]
struct Demon;

/// Auto-tag meshes with "eye" in their name with RenderOnTop
fn tag_demon_eyes(
    mut commands: Commands,
    query: Query<(Entity, &Name), Without<RenderOnTop>>,
) {
    for (entity, name) in &query {
        if name.as_str().to_lowercase().contains("eye") {
            commands.entity(entity).insert(RenderOnTop::default());
        }
    }
}

#[derive(Component)]
struct SpeechBubbleRoot;

#[derive(Component)]
struct SpeechBubble {
    world_offset: Vec3,
    screen_offset: Vec2,
}

const SPEECH_BUBBLE_WORLD_OFFSET: Vec3 = Vec3::new(0.0, 2.2, 0.0);
const SPEECH_BUBBLE_SCREEN_OFFSET: Vec2 = Vec2::new(0.0, 0.0);

#[derive(Resource, Debug, Clone)]
struct ChromaticAberrationUi {
    k_rgb: [f32; 3],
}

impl Default for ChromaticAberrationUi {
    fn default() -> Self {
        Self {
            // Matches the shader's default: out = current + k * (current - left)
            k_rgb: [0.2, -0.5, -1.2],
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct LineBoilUi {
    intensity: f32,
    frame_rate: f32,
    noise_frequency: f32,
}

impl Default for LineBoilUi {
    fn default() -> Self {
        Self {
            intensity: 0.008,
            frame_rate: 8.0,
            noise_frequency: 6.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
struct DepthBiasUi {
    bias: f32,
}

impl Default for DepthBiasUi {
    fn default() -> Self {
        Self {
            bias: 100000.0,
        }
    }
}

#[derive(Component)]
struct Skybox;

#[derive(Resource, Debug, Clone)]
struct SkyboxTintUi {
    tint_rgb: [f32; 3],
}

impl Default for SkyboxTintUi {
    fn default() -> Self {
        Self {
            tint_rgb: [1.0, 1.0, 1.0], // No tint by default
        }
    }
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

fn orbit_psx_camera(time: Res<Time>, mut query: Query<&mut Transform, With<OrbitingCamera>>) {
    let angle = time.elapsed_secs() * 0.3;
    let x_radius = 0.4;
    let y_radius = 0.25;
    for mut transform in &mut query {
        let z = transform.translation.z;
        transform.translation = Vec3::new(angle.cos() * x_radius, angle.sin() * y_radius + 2., 10.);
    }
}

fn freecam_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut query: Query<(&mut Transform, &mut FreeCam)>,
) {
    for (mut transform, mut freecam) in &mut query {
        // Mouse look
        for event in mouse_motion.read() {
            freecam.yaw -= event.delta.x * freecam.sensitivity;
            freecam.pitch -= event.delta.y * freecam.sensitivity;
            freecam.pitch = freecam.pitch.clamp(-1.5, 1.5);
        }
        transform.rotation = Quat::from_euler(EulerRot::YXZ, freecam.yaw, freecam.pitch, 0.0);

        // WASD movement
        let mut direction = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            direction += *transform.forward();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            direction += *transform.back();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            direction += *transform.left();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            direction += *transform.right();
        }
        // Up/Down with Space and Shift
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * freecam.speed * time.delta_secs();
    }
}

fn sync_speech_bubble(
    mut commands: Commands,
    mut bubble_query: Query<(&mut Node, &ComputedNode, &mut Visibility, &SpeechBubble)>,
    bubble_root_query: Query<Entity, With<SpeechBubbleRoot>>,
    demon_query: Query<&GlobalTransform, With<Demon>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PsxCamera>>,
    final_camera_query: Query<(Entity, &Camera), With<FinalCameraTag>>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(demon_transform) = demon_query.single() else {
        return;
    };
    let Ok((final_camera_entity, final_camera)) = final_camera_query.single() else {
        return;
    };
    let Some(viewport_rect) = final_camera.logical_viewport_rect() else {
        return;
    };

    if let Ok(root_entity) = bubble_root_query.single() {
        commands
            .entity(root_entity)
            .try_insert(UiTargetCamera(final_camera_entity));
    }

    for (mut node, computed, mut visibility, bubble) in &mut bubble_query {
        let world_position = demon_transform.translation() + bubble.world_offset;
        let Some(ndc) = camera.world_to_ndc(camera_transform, world_position) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if ndc.z < 0.0 || ndc.z > 1.0 {
            *visibility = Visibility::Hidden;
            continue;
        }

        let viewport_size = viewport_rect.size();
        let mut viewport_position = (ndc.truncate() + Vec2::ONE) * 0.5 * viewport_size;
        viewport_position.y = viewport_size.y - viewport_position.y;
        viewport_position += viewport_rect.min;

        let logical_size = computed.size() * computed.inverse_scale_factor();
        let mut left = viewport_position.x + bubble.screen_offset.x;
        let mut top = viewport_position.y + bubble.screen_offset.y;

        if logical_size.x > 0.0 && logical_size.y > 0.0 {
            left -= logical_size.x * 0.5;
            top -= logical_size.y;
        }

        node.position_type = PositionType::Absolute;
        node.left = Val::Px(left);
        node.top = Val::Px(top);
        *visibility = Visibility::Visible;
    }
}

fn chromatic_aberration_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<ChromaticAberrationUi>,
    mut resolution_scale: ResMut<ResolutionScale>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    egui::Window::new("PSX Dither Tweaks").show(ctx, |ui| {
        ui.label("Chromatic aberration tint (RGB)");
        ui.label("0 disables a channel; affects only edges (tap difference).");
        ui.add_space(8.0);

        ui.add(egui::Slider::new(&mut state.k_rgb[0], -2.0..=2.0).text("R"));
        ui.add(egui::Slider::new(&mut state.k_rgb[1], -2.0..=2.0).text("G"));
        ui.add(egui::Slider::new(&mut state.k_rgb[2], -2.0..=2.0).text("B"));

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label("Resolution Scale");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut resolution_scale.divisor, 1, "Full");
            ui.selectable_value(&mut resolution_scale.divisor, 2, "1/2");
            ui.selectable_value(&mut resolution_scale.divisor, 4, "1/4");
        });

        ui.add_space(8.0);
        if ui.button("Reset").clicked() {
            *state = ChromaticAberrationUi::default();
            *resolution_scale = ResolutionScale::default();
        }
    });
}

fn apply_chromatic_aberration_to_material(
    state: Res<ChromaticAberrationUi>,
    render_images: Query<&MeshMaterial2d<PsxDitherMaterial>, With<RenderImage>>,
    mut materials: ResMut<Assets<PsxDitherMaterial>>,
) {
    if !state.is_changed() {
        return;
    }

    for material_handle in &render_images {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        material.chroma_k = Vec4::new(state.k_rgb[0], state.k_rgb[1], state.k_rgb[2], 0.0);
    }
}

fn line_boil_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<LineBoilUi>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    egui::Window::new("Line Boil").show(ctx, |ui| {
        ui.label("Screen-space vertex displacement");
        ui.add_space(8.0);

        ui.add(egui::Slider::new(&mut state.intensity, 0.0..=0.1).text("Intensity"));
        ui.add(egui::Slider::new(&mut state.frame_rate, 1.0..=30.0).text("Frame Rate"));
        ui.add(egui::Slider::new(&mut state.noise_frequency, 0.5..=20.0).text("Noise Freq"));

        ui.add_space(8.0);
        if ui.button("Reset").clicked() {
            *state = LineBoilUi::default();
        }
    });
}

fn apply_line_boil_ui(
    state: Res<LineBoilUi>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    if !state.is_changed() {
        return;
    }

    for (_, material) in materials.iter_mut() {
        material.extension.settings.intensity = state.intensity;
        material.extension.settings.frame_rate = state.frame_rate;
        material.extension.settings.noise_frequency = state.noise_frequency;
    }
}

fn depth_bias_ui(mut contexts: EguiContexts, mut state: ResMut<DepthBiasUi>) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    egui::Window::new("Depth Bias (Eyes)").show(ctx, |ui| {
        ui.label("Controls eye render-on-top effect");
        ui.add_space(8.0);

        ui.add(egui::Slider::new(&mut state.bias, 0.0..=500000.0).text("Bias"));

        ui.add_space(8.0);
        if ui.button("Reset").clicked() {
            *state = DepthBiasUi::default();
        }
    });
}

fn skybox_tint_ui(mut contexts: EguiContexts, mut state: ResMut<SkyboxTintUi>) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    egui::Window::new("Skybox Tint").show(ctx, |ui| {
        ui.label("Multiplies skybox color by tint");
        ui.add_space(8.0);

        ui.add(egui::Slider::new(&mut state.tint_rgb[0], 0.0..=2.0).text("R"));
        ui.add(egui::Slider::new(&mut state.tint_rgb[1], 0.0..=2.0).text("G"));
        ui.add(egui::Slider::new(&mut state.tint_rgb[2], 0.0..=2.0).text("B"));

        ui.add_space(8.0);
        if ui.button("Reset").clicked() {
            *state = SkyboxTintUi::default();
        }
    });
}

fn apply_skybox_tint_ui(
    state: Res<SkyboxTintUi>,
    mut skybox_query: Query<&mut SkyboxTint, With<Skybox>>,
) {
    if !state.is_changed() {
        return;
    }

    for mut skybox_tint in &mut skybox_query {
        skybox_tint.settings.tint = Vec4::new(
            state.tint_rgb[0],
            state.tint_rgb[1],
            state.tint_rgb[2],
            1.0,
        );
    }
}

fn apply_depth_bias_ui(
    state: Res<DepthBiasUi>,
    mut render_on_top_query: Query<&mut RenderOnTop>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    if !state.is_changed() {
        return;
    }

    // Update RenderOnTop components
    for mut rot in &mut render_on_top_query {
        rot.bias = state.bias;
    }

    // Update materials that already have depth_bias set
    for (_, material) in materials.iter_mut() {
        if material.base.depth_bias > 0.0 {
            material.base.depth_bias = state.bias;
        }
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
