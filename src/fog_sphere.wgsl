#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::mesh_view_bindings::{globals, view}
#import bevy_pbr::prepass_utils
#import bevy_render::instance_index::get_instance_index

struct FogSphereMaterial {
    fog_color: vec4<f32>,
    sky_color: vec4<f32>,
    fog_data: vec4<f32>,
    depth_range: vec2<f32>,
    noise_data: vec2<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: FogSphereMaterial;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_dir: vec3<f32>,
};

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_from_local = get_world_from_local(input.instance_index);
    let local_position = vec4<f32>(input.position, 1.0);
    let world_position = world_from_local * local_position;
    output.clip_position = mesh_position_local_to_clip(world_from_local, local_position);
    output.world_dir = world_position.xyz - view.world_position;
    return output;
}

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_dir: vec3<f32>,
};

fn scene_depth(frag_coord: vec4<f32>) -> f32 {
#ifdef DEPTH_PREPASS
    return prepass_utils::prepass_depth(frag_coord, 0u);
#else
    return 1.0;
#endif
}

fn hash_noise(p: vec2<f32>) -> f32 {
    let value = sin(dot(p, vec2<f32>(12.9898, 78.233)) + globals.time * 0.03) * 43758.5453;
    return fract(value);
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let dir = normalize(input.world_dir);
    let horizon =
        1.0 - smoothstep(material.fog_data.x - material.fog_data.y, material.fog_data.x + material.fog_data.y, dir.y);
    let depth_sample = smoothstep(material.depth_range.x, material.depth_range.y, scene_depth(input.frag_coord));
    let wobble = hash_noise(dir.xz * material.noise_data.x);
    let clouds = mix(1.0, wobble, material.noise_data.y);
    let fog_strength = clamp(horizon * depth_sample * clouds, 0.0, 1.0);
    let base_color = mix(material.sky_color.rgb, material.fog_color.rgb, fog_strength);
    let final_color = mix(material.sky_color.rgb, base_color, fog_strength + material.fog_data.w * (1.0 - depth_sample));
    let alpha = fog_strength * material.fog_data.z;
 //   return vec4<f32>(final_color, alpha);
    return vec4<f32>(final_color.r, depth_sample, horizon, 1.);
 //   return vec4<f32>(1., 0., 0., 1.);
}
