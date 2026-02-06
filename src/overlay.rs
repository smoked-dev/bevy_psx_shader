//! Anime-style overlay rendering plugin.
//!
//! Provides `OverlayMaterial` with depth bias support for "always on top" effects.
//! Useful for rendering eyes/eyebrows through hair, like in anime.
//!
//! # Usage
//!
//! ```rust,ignore
//! app.add_plugins(OverlayPlugin);
//!
//! // Add RenderOnTop to any entity with PsxMaterial - it will be auto-swapped
//! commands.entity(eyes_entity).insert(RenderOnTop::default());
//! ```

use bevy::{
    pbr::{ExtendedMaterial, MaterialPlugin, MeshMaterial3d},
    prelude::*,
    reflect::TypePath,
    render::render_resource::*,
    shader::ShaderRef,
};

use crate::{
    line_boil::LineBoilMaterial,
    material::{PsxMaterial, PSX_FRAG_SHADER_HANDLE, PSX_VERT_SHADER_HANDLE},
};

/// Plugin for anime-style overlay rendering.
/// Provides OverlayMaterial with depth_bias for "always on top" effects.
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OverlayMaterial>::default());
        app.add_systems(
            PostUpdate,
            (
                swap_to_overlay_material,
                swap_standard_to_overlay_material,
                set_lineboil_depth_bias,
            ),
        );
    }
}

/// Marker component for meshes that should render in front of siblings.
///
/// When added to an entity with `PsxMaterial`, the material is automatically
/// swapped to `OverlayMaterial` with the specified depth bias.
///
/// Depth bias works relative to the mesh's actual depth, so:
/// - Demon A's eyes render in front of Demon A's hair
/// - Demon B's eyes render in front of Demon B's hair
/// - But proper depth sorting between demons is preserved
#[derive(Component, Clone, Copy)]
pub struct RenderOnTop {
    /// Depth bias value. Higher = renders closer to camera.
    /// Default: 10000.0
    pub bias: f32,
}

impl Default for RenderOnTop {
    fn default() -> Self {
        Self { bias: 100000.0 }
    }
}

impl RenderOnTop {
    pub fn new(bias: f32) -> Self {
        Self { bias }
    }
}

/// Material with depth_bias support for overlay rendering.
/// Uses the same PSX shaders but adds depth bias to render in front of other meshes.
#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct OverlayMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub fog_color: LinearRgba,
    #[uniform(0)]
    pub snap_amount: f32,
    #[uniform(0)]
    pub fog_distance: Vec2,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    /// Depth bias - higher values render closer to camera
    pub depth_bias: f32,
}

impl Default for OverlayMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::WHITE,
            fog_color: LinearRgba::WHITE,
            snap_amount: 5.0,
            fog_distance: Vec2::new(25.0, 75.0),
            color_texture: None,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 1000.0,
        }
    }
}

impl Material for OverlayMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PSX_FRAG_SHADER_HANDLE)
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(PSX_VERT_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}

/// System: When RenderOnTop is added, swap PsxMaterial -> OverlayMaterial
fn swap_to_overlay_material(
    mut commands: Commands,
    query: Query<
        (Entity, &MeshMaterial3d<PsxMaterial>, &RenderOnTop),
        Added<RenderOnTop>,
    >,
    psx_materials: Res<Assets<PsxMaterial>>,
    mut overlay_materials: ResMut<Assets<OverlayMaterial>>,
) {
    for (entity, psx_handle, render_on_top) in &query {
        let Some(psx_mat) = psx_materials.get(psx_handle) else {
            continue;
        };
        let overlay_mat = overlay_materials.add(OverlayMaterial {
            color: psx_mat.color,
            fog_color: psx_mat.fog_color,
            snap_amount: psx_mat.snap_amount,
            fog_distance: psx_mat.fog_distance,
            color_texture: psx_mat.color_texture.clone(),
            alpha_mode: psx_mat.alpha_mode,
            depth_bias: render_on_top.bias,
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<PsxMaterial>>()
            .insert(MeshMaterial3d(overlay_mat));
    }
}

/// System: When RenderOnTop is added, swap StandardMaterial -> OverlayMaterial
/// This handles GLTF-loaded meshes which use StandardMaterial by default.
fn swap_standard_to_overlay_material(
    mut commands: Commands,
    query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>, &RenderOnTop),
        Added<RenderOnTop>,
    >,
    std_materials: Res<Assets<StandardMaterial>>,
    mut overlay_materials: ResMut<Assets<OverlayMaterial>>,
) {
    for (entity, std_handle, render_on_top) in &query {
        let Some(std_mat) = std_materials.get(std_handle) else {
            continue;
        };
        let overlay_mat = overlay_materials.add(OverlayMaterial {
            color: std_mat.base_color.into(),
            fog_color: LinearRgba::WHITE,
            snap_amount: 5.0,
            fog_distance: Vec2::new(25.0, 75.0),
            color_texture: std_mat.base_color_texture.clone(),
            alpha_mode: std_mat.alpha_mode,
            depth_bias: render_on_top.bias,
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(overlay_mat));
    }
}

/// System: When RenderOnTop is added to an entity with LineBoil material,
/// create a new material with depth_bias set and swap it.
fn set_lineboil_depth_bias(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &MeshMaterial3d<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>,
            &RenderOnTop,
        ),
        Added<RenderOnTop>,
    >,
    mut lineboil_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    // Collect entities to process first to avoid borrow issues
    let to_process: Vec<_> = query
        .iter()
        .map(|(e, h, r)| (e, h.0.clone(), r.bias))
        .collect();

    for (entity, handle, bias) in to_process {
        let Some(lb_mat) = lineboil_materials.get(&handle).cloned() else {
            continue;
        };
        let mut new_base = lb_mat.base.clone();
        new_base.depth_bias = bias;

        let new_mat = lineboil_materials.add(ExtendedMaterial {
            base: new_base,
            extension: lb_mat.extension.clone(),
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>()
            .insert(MeshMaterial3d(new_mat));
    }
}
