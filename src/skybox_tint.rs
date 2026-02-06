//! Skybox Tint Effect Plugin
//!
//! Applies a color tint to skybox meshes by multiplying the base color by a tint value.

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

/// Shader handle for the skybox tint fragment shader
pub const SKYBOX_TINT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("a1b2c3d4-e5f6-7890-abcd-ef1234567890");

/// Plugin that adds skybox tint effect support.
///
/// Add this plugin to your app, then add the [`SkyboxTint`] component to any entity
/// with a scene to apply the tint to all its meshes.
pub struct SkyboxTintPlugin;

impl Plugin for SkyboxTintPlugin {
    fn build(&self, app: &mut App) {
        // Register the extended material
        app.add_plugins(
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>::default(),
        );

        // Load the shader
        load_internal_asset!(
            app,
            SKYBOX_TINT_SHADER_HANDLE,
            "skybox_tint.wgsl",
            Shader::from_wgsl
        );

        // Add systems
        app.add_systems(
            Update,
            (
                apply_skybox_tint_to_marked_entities,
                cleanup_old_materials.after(apply_skybox_tint_to_marked_entities),
                sync_tint_to_materials,
            ),
        );
    }
}

/// Settings for the skybox tint effect.
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct SkyboxTintSettings {
    /// The tint color to multiply the base color by.
    /// RGBA where (1,1,1,1) means no change.
    pub tint: Vec4,
}

impl Default for SkyboxTintSettings {
    fn default() -> Self {
        Self {
            tint: Vec4::ONE, // No tint by default (white = multiply by 1)
        }
    }
}

/// The skybox tint material extension.
///
/// This extends `StandardMaterial` with a color tint multiplier.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct SkyboxTintMaterial {
    #[uniform(100)]
    pub settings: SkyboxTintSettings,
}

impl Default for SkyboxTintMaterial {
    fn default() -> Self {
        Self {
            settings: SkyboxTintSettings::default(),
        }
    }
}

impl MaterialExtension for SkyboxTintMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SKYBOX_TINT_SHADER_HANDLE)
    }
}

/// Marker component to apply skybox tint effect to an entity and its mesh children.
///
/// Add this component to an entity (typically a glTF scene root) to apply the
/// tint effect to all meshes within its hierarchy.
#[derive(Component, Default, Clone)]
pub struct SkyboxTint {
    pub settings: SkyboxTintSettings,
}

impl SkyboxTint {
    /// Create a new SkyboxTint with default settings (no tint).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific tint color.
    pub fn with_color(mut self, color: Color) -> Self {
        let linear: LinearRgba = color.into();
        self.settings.tint = Vec4::new(linear.red, linear.green, linear.blue, linear.alpha);
        self
    }

    /// Create with RGB values (0.0 to 1.0).
    pub fn with_rgb(mut self, r: f32, g: f32, b: f32) -> Self {
        self.settings.tint = Vec4::new(r, g, b, 1.0);
        self
    }

    /// Set the tint from a Vec4.
    pub fn with_tint(mut self, tint: Vec4) -> Self {
        self.settings.tint = tint;
        self
    }
}

/// Marker component to track meshes that have already been processed.
#[derive(Component)]
struct SkyboxTintApplied;

/// Cleanup system that removes any leftover StandardMaterial from entities
/// that have been processed (have SkyboxTintApplied marker).
fn cleanup_old_materials(
    mut commands: Commands,
    query: Query<Entity, (With<SkyboxTintApplied>, With<MeshMaterial3d<StandardMaterial>>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

/// System that syncs the tint from SkyboxTint components to all their child materials.
fn sync_tint_to_materials(
    root_query: Query<(Entity, &SkyboxTint), Changed<SkyboxTint>>,
    children_query: Query<&Children>,
    tint_applied_query: Query<&MeshMaterial3d<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>, With<SkyboxTintApplied>>,
    mut tint_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>>,
) {
    for (root_entity, skybox_tint) in root_query.iter() {
        sync_tint_recursive(
            root_entity,
            &skybox_tint.settings,
            &children_query,
            &tint_applied_query,
            &mut tint_materials,
        );
    }
}

fn sync_tint_recursive(
    entity: Entity,
    settings: &SkyboxTintSettings,
    children_query: &Query<&Children>,
    tint_applied_query: &Query<&MeshMaterial3d<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>, With<SkyboxTintApplied>>,
    tint_materials: &mut Assets<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>,
) {
    // Update this entity's material if it has one
    if let Ok(mat_handle) = tint_applied_query.get(entity) {
        if let Some(material) = tint_materials.get_mut(&mat_handle.0) {
            material.extension.settings = *settings;
        }
    }

    // Recurse into children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            sync_tint_recursive(child, settings, children_query, tint_applied_query, tint_materials);
        }
    }
}

/// System that replaces StandardMaterial with SkyboxTintMaterial on entities marked with SkyboxTint.
/// Runs every frame to catch meshes that spawn after the SkyboxTint component is added (e.g., glTF scenes).
fn apply_skybox_tint_to_marked_entities(
    mut commands: Commands,
    root_query: Query<(Entity, &SkyboxTint)>,
    children_query: Query<&Children>,
    mesh_query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<SkyboxTintApplied>,
    >,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut tint_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>>,
) {
    for (root_entity, skybox_tint) in root_query.iter() {
        traverse_and_replace_materials(
            root_entity,
            skybox_tint,
            &children_query,
            &mesh_query,
            &standard_materials,
            &mut tint_materials,
            &mut commands,
        );
    }
}

fn traverse_and_replace_materials(
    entity: Entity,
    skybox_tint: &SkyboxTint,
    children_query: &Query<&Children>,
    mesh_query: &Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<SkyboxTintApplied>,
    >,
    standard_materials: &Assets<StandardMaterial>,
    tint_materials: &mut Assets<ExtendedMaterial<StandardMaterial, SkyboxTintMaterial>>,
    commands: &mut Commands,
) {
    // If this entity has a mesh with StandardMaterial that hasn't been processed, replace it
    if let Ok((_, mat_handle)) = mesh_query.get(entity) {
        if let Some(std_mat) = standard_materials.get(&mat_handle.0) {
            let extended = ExtendedMaterial {
                base: std_mat.clone(),
                extension: SkyboxTintMaterial {
                    settings: skybox_tint.settings,
                },
            };
            let new_handle = tint_materials.add(extended);

            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(new_handle))
                .insert(SkyboxTintApplied);
        }
    }

    // Recurse into children
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            traverse_and_replace_materials(
                child,
                skybox_tint,
                children_query,
                mesh_query,
                standard_materials,
                tint_materials,
                commands,
            );
        }
    }
}
