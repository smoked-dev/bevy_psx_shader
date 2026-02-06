// Skybox Tint Fragment Shader (Unlit)
// Multiplies the base texture color by a tint color, no lighting applied.

#import bevy_pbr::{
    mesh_view_bindings::view,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_bindings,
}

struct SkyboxTintSettings {
    tint: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> skybox_tint: SkyboxTintSettings;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var out: FragmentOutput;

    // Sample the base color texture if available, otherwise use vertex color or white
#ifdef VERTEX_UVS
    let base_color = textureSample(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, in.uv);
#else
    let base_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
#endif

    // Multiply by material base color (from StandardMaterial)
    let material_color = pbr_bindings::material.base_color;

    // Final color = texture * material base color * tint (unlit, no lighting)
    out.color = base_color * material_color * skybox_tint.tint;

    return out;
}
