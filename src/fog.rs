use bevy::{
    asset::load_internal_asset,
    color::LinearRgba,
    math::primitives::Sphere,
    pbr::{Material, MeshMaterial3d, NotShadowCaster},
    prelude::*,
    render::{
        mesh::Mesh3d,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, Shader, ShaderRef,
            SpecializedMeshPipelineError,
        },
        view::NoFrustumCulling,
    },
};

use crate::camera::PsxCamera;

pub const FOG_SPHERE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(257512391659684203498);

const DEFAULT_RADIUS: f32 = 500.0;

#[derive(Resource)]
pub struct FogSphereSettings {
    pub radius: f32,
}

impl Default for FogSphereSettings {
    fn default() -> Self {
        Self {
            radius: DEFAULT_RADIUS,
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct FogSphereMaterial {
    #[uniform(0)]
    pub fog_color: Vec4,
    #[uniform(0)]
    pub sky_color: Vec4,
    /// x = horizon height, y = horizon falloff, z = max alpha, w = base sky mix
    #[uniform(0)]
    pub fog_data: Vec4,
    /// x = start depth, y = end depth
    #[uniform(0)]
    pub depth_range: Vec2,
    /// x = noise scale, y = noise strength
    #[uniform(0)]
    pub noise_data: Vec2,
}

impl Default for FogSphereMaterial {
    fn default() -> Self {
        Self {
            fog_color: color_to_vec4(Color::linear_rgb(0.95, 0.97, 1.0)),
            sky_color: color_to_vec4(Color::linear_rgb(0.15, 0.2, 0.4)),
            fog_data: Vec4::new(0.02, 0.25, 0.85, 0.2),
            depth_range: Vec2::new(0.6, 0.98),
            noise_data: Vec2::new(1.35, 0.4),
        }
    }
}

impl Material for FogSphereMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(FOG_SPHERE_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(FOG_SPHERE_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(state) = descriptor.depth_stencil.as_mut() {
            state.depth_write_enabled = false;
        }
        descriptor.primitive.cull_mode = Some(Face::Front);
        descriptor.primitive.unclipped_depth = true;
        Ok(())
    }
}

#[derive(Component)]
pub struct FogSphere {
    pub camera: Entity,
    pub radius: f32,
    pub material: Handle<FogSphereMaterial>,
}

pub struct FogPlugin;

impl Plugin for FogPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FOG_SPHERE_SHADER_HANDLE,
            "fog_sphere.wgsl",
            Shader::from_wgsl
        );

        {
            let world = app.world_mut();
            if world.get_resource::<FogSphereSettings>().is_none() {
                world.insert_resource(FogSphereSettings::default());
            }
        }
        app
        .add_plugins(MaterialPlugin::<FogSphereMaterial> {
            shadows_enabled: false,
            ..default()
        })
        .add_systems(
            PostUpdate,
            (
                spawn_fog_spheres,
                update_fog_spheres,
                cleanup_orphaned_spheres,
            )
                .chain(),
        );
    }
}

fn spawn_fog_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FogSphereMaterial>>,
    settings: Res<FogSphereSettings>,
    cameras: Query<(Entity, &GlobalTransform), Added<PsxCamera>>,
) {
    for (camera_entity, transform) in cameras.iter() {
        let mesh = meshes.add(Mesh::from(Sphere::new(1.0)));
        let material = materials.add(FogSphereMaterial::default());
        let fog_transform =
            Transform::from_translation(transform.translation()).with_scale(Vec3::splat(settings.radius));

        commands.spawn((
            FogSphere {
                camera: camera_entity,
                radius: settings.radius,
                material: material.clone(),
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            NoFrustumCulling,
            NotShadowCaster,
            fog_transform,
            Visibility::Visible,
            Name::new("PSX Fog Sphere"),
        ));
    }
}

fn update_fog_spheres(
    mut spheres: Query<(&FogSphere, &mut Transform)>,
    cameras: Query<&GlobalTransform>,
) {
    for (fog, mut transform) in spheres.iter_mut() {
        if let Ok(camera_transform) = cameras.get(fog.camera) {
            transform.translation = camera_transform.translation();
            transform.scale = Vec3::splat(fog.radius);
        }
    }
}

fn cleanup_orphaned_spheres(
    mut commands: Commands,
    spheres: Query<(Entity, &FogSphere)>,
    cameras: Query<(), With<PsxCamera>>,
) {
    for (entity, fog) in spheres.iter() {
        if cameras.get(fog.camera).is_err() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn color_to_vec4(color: Color) -> Vec4 {
    LinearRgba::from(color).to_vec4()
}
