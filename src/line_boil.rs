//! Line Boil Effect Plugin
//!
//! Applies a classic cartoon "line boil" effect via turbulent vertex displacement.
//! The effect creates a hand-drawn animation look by jittering vertices at fixed frame intervals.

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

/// Shader handle for the line boil vertex shader
pub const LINE_BOIL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("89237458-9234-4589-a3ab-cdef12345678");

/// Plugin that adds line boil effect support.
///
/// Add this plugin to your app, then add the [`LineBoil`] component to any entity
/// with a glTF scene to apply the effect to all its meshes.
pub struct LineBoilPlugin;

impl Plugin for LineBoilPlugin {
    fn build(&self, app: &mut App) {
        // Register the extended material
        app.add_plugins(
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>::default(),
        );

        // Load the shader
        load_internal_asset!(
            app,
            LINE_BOIL_SHADER_HANDLE,
            "line_boil.wgsl",
            Shader::from_wgsl
        );

        // Add systems - cleanup runs after apply to ensure old materials are removed
        app.add_systems(
            Update,
            (
                apply_line_boil_to_marked_entities,
                cleanup_old_materials.after(apply_line_boil_to_marked_entities),
                update_line_boil_time,
            ),
        );
    }
}

/// System that updates the time uniform in all line boil materials.
fn update_line_boil_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    let current_time = time.elapsed_secs();
    for (_, material) in materials.iter_mut() {
        material.extension.settings.time = current_time;
    }
}

/// Settings for the line boil vertex displacement effect.
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct LineBoilSettings {
    /// Displacement intensity (how far vertices move).
    /// Higher values = more aggressive jitter.
    /// Recommended: 0.02-0.1 for subtle, 0.1-0.3 for aggressive.
    pub intensity: f32,

    /// Frames per second for time quantization.
    /// Lower values = more "held" frames (classic animation look).
    /// Classic animation: 4-8 fps. Smooth: 12-24 fps.
    pub frame_rate: f32,

    /// Noise frequency - controls turbulence scale.
    /// Higher values = more chaotic per-vertex variation.
    pub noise_frequency: f32,

    /// Seed offset for noise variation between entities.
    pub seed: f32,

    /// Current time (updated by system each frame).
    /// This is internal - users shouldn't set this directly.
    #[doc(hidden)]
    pub time: f32,
}

impl Default for LineBoilSettings {
    fn default() -> Self {
        Self {
            intensity: 0.02,  // Small values for screen-space displacement
            frame_rate: 6.0,
            noise_frequency: 8.0,  // Higher frequency = more waves across the model
            seed: 0.0,
            time: 0.0,
        }
    }
}

/// The line boil material extension.
///
/// This extends `StandardMaterial` with vertex displacement for the line boil effect.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct LineBoilMaterial {
    #[uniform(100)]
    pub settings: LineBoilSettings,
}

impl Default for LineBoilMaterial {
    fn default() -> Self {
        Self {
            settings: LineBoilSettings::default(),
        }
    }
}

impl MaterialExtension for LineBoilMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(LINE_BOIL_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}

/// Marker component to apply line boil effect to an entity and its mesh children.
///
/// Add this component to an entity (typically a glTF scene root) to apply the
/// line boil effect to all meshes within its hierarchy.
#[derive(Component, Default, Clone)]
pub struct LineBoil {
    pub settings: LineBoilSettings,
}

impl LineBoil {
    /// Create a new LineBoil with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the displacement intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.settings.intensity = intensity;
        self
    }

    /// Set the frame rate for time quantization.
    pub fn with_frame_rate(mut self, fps: f32) -> Self {
        self.settings.frame_rate = fps;
        self
    }

    /// Set the noise frequency.
    pub fn with_noise_frequency(mut self, freq: f32) -> Self {
        self.settings.noise_frequency = freq;
        self
    }

    /// Set the noise seed for variation.
    pub fn with_seed(mut self, seed: f32) -> Self {
        self.settings.seed = seed;
        self
    }

    /// Create with aggressive jitter preset.
    pub fn aggressive() -> Self {
        Self {
            settings: LineBoilSettings {
                intensity: 0.04,  // Noticeable but not extreme
                frame_rate: 4.0,
                noise_frequency: 12.0,  // More waves for chaotic look
                seed: 0.0,
                time: 0.0,
            },
        }
    }

    /// Create with subtle boil preset.
    pub fn subtle() -> Self {
        Self {
            settings: LineBoilSettings {
                intensity: 0.008,  // Very subtle wobble
                frame_rate: 8.0,
                noise_frequency: 6.0,  // Moderate wave count
                seed: 0.0,
                time: 0.0,
            },
        }
    }
}

/// Marker component to track meshes that have already been processed.
#[derive(Component)]
struct LineBoilApplied;

/// Cleanup system that removes any leftover StandardMaterial from entities
/// that have been processed (have LineBoilApplied marker).
/// This handles edge cases where the remove command might not have fully processed.
fn cleanup_old_materials(
    mut commands: Commands,
    query: Query<Entity, (With<LineBoilApplied>, With<MeshMaterial3d<StandardMaterial>>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

/// System that replaces StandardMaterial with LineBoilMaterial on entities marked with LineBoil.
/// Runs every frame to catch meshes that spawn after the LineBoil component is added (e.g., glTF scenes).
fn apply_line_boil_to_marked_entities(
    mut commands: Commands,
    root_query: Query<(Entity, &LineBoil)>,
    children_query: Query<&Children>,
    mesh_query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<LineBoilApplied>,
    >,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut line_boil_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    for (root_entity, line_boil) in root_query.iter() {
        traverse_and_replace_materials(
            root_entity,
            line_boil,
            &children_query,
            &mesh_query,
            &standard_materials,
            &mut line_boil_materials,
            &mut commands,
        );
    }
}

fn traverse_and_replace_materials(
    entity: Entity,
    line_boil: &LineBoil,
    children_query: &Query<&Children>,
    mesh_query: &Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<LineBoilApplied>,
    >,
    standard_materials: &Assets<StandardMaterial>,
    line_boil_materials: &mut Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>,
    commands: &mut Commands,
) {
    // If this entity has a mesh with StandardMaterial that hasn't been processed, replace it
    if let Ok((_, mat_handle)) = mesh_query.get(entity) {
        if let Some(std_mat) = standard_materials.get(&mat_handle.0) {
            let extended = ExtendedMaterial {
                base: std_mat.clone(),
                extension: LineBoilMaterial {
                    settings: line_boil.settings,
                },
            };
            let new_handle = line_boil_materials.add(extended);

            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(new_handle))
                .insert(LineBoilApplied);
        }
    }

    // Recurse into children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            traverse_and_replace_materials(
                child,
                line_boil,
                children_query,
                mesh_query,
                standard_materials,
                line_boil_materials,
                commands,
            );
        }
    }
}
