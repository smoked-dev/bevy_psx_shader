// Line Boil Vertex Shader
// Creates a classic cartoon "line boil" effect via turbulent vertex displacement.
// Displacement is quantized to discrete frame intervals for that hand-drawn animation look.

#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

// Line boil settings uniform (time is passed through material uniform)
struct LineBoilSettings {
    intensity: f32,
    frame_rate: f32,
    noise_frequency: f32,
    seed: f32,
    time: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> line_boil: LineBoilSettings;

// ============================================================================
// Smooth value noise functions (spatially coherent - nearby vertices move together)
// ============================================================================

// Simple hash function for 3D input -> single float
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

// Smooth interpolation (attempt #2 with smoothstep)
fn smooth_interp(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

// 3D value noise with smooth trilinear interpolation
// This ensures nearby points get similar values (no vertex clipping)
fn value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Smooth interpolation weights
    let u = vec3<f32>(smooth_interp(f.x), smooth_interp(f.y), smooth_interp(f.z));

    // Hash at 8 corners of the cell
    let n000 = hash31(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash31(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash31(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash31(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash31(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash31(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash31(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash31(i + vec3<f32>(1.0, 1.0, 1.0));

    // Trilinear interpolation
    let n00 = mix(n000, n100, u.x);
    let n10 = mix(n010, n110, u.x);
    let n01 = mix(n001, n101, u.x);
    let n11 = mix(n011, n111, u.x);
    let n0 = mix(n00, n10, u.y);
    let n1 = mix(n01, n11, u.y);
    return mix(n0, n1, u.z) * 2.0 - 1.0;  // Return -1 to 1
}

// Quantize time to create frame-held effect
fn quantize_time(time: f32, fps: f32) -> f32 {
    return floor(time * fps);
}

// Smooth 3D displacement vector - nearby vertices get similar displacement
fn smooth_turbulent_noise(pos: vec3<f32>, time_q: f32, seed: f32) -> vec3<f32> {
    let p = pos * line_boil.noise_frequency + seed;
    let t = time_q;

    // Sample smooth noise for each axis with different offsets
    // This creates a coherent wave-like displacement field
    return vec3<f32>(
        value_noise_3d(p + vec3<f32>(t * 1.0, 0.0, 0.0)),
        value_noise_3d(p + vec3<f32>(0.0, t * 1.3, 100.0)),
        value_noise_3d(p + vec3<f32>(200.0, 0.0, t * 0.7))
    );
}

// ============================================================================
// Vertex shader entry point
// ============================================================================

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Handle morphing if enabled
#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

    // Handle skinning if enabled
#ifdef SKINNED
    var world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights, vertex.instance_index);
#else
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
#endif

    // Get world position
    var world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));

    // Get world normal for displacement direction
#ifdef VERTEX_NORMALS
    var world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
#else
    var world_normal = vec3<f32>(0.0, 1.0, 0.0);
#endif

    // ========================================================================
    // LINE BOIL DISPLACEMENT (Screen-space for hand-drawn effect)
    // ========================================================================

    // Transform to clip space first
    var clip_position = position_world_to_clip(world_position.xyz);

    // Quantize time to create frame-held effect (classic animation look)
    let time_quantized = quantize_time(line_boil.time, line_boil.frame_rate);

    // Generate smooth displacement based on world position (for spatial coherence)
    // but apply it in screen space X/Y for that hand-drawn 2D wobble
    let noise = smooth_turbulent_noise(world_position.xyz, time_quantized, line_boil.seed);

    // Displace in screen space (X and Y only) - like lines drawn on paper wobbling
    // Scale by w to keep displacement consistent regardless of depth
    clip_position.x += noise.x * line_boil.intensity * clip_position.w;
    clip_position.y += noise.y * line_boil.intensity * clip_position.w;

    // DEBUG: Move 2x to the left to see which demon is which
    clip_position.x -= 0.25 * clip_position.w;

    // ========================================================================

    out.position = clip_position;
    out.world_position = world_position;

#ifdef VERTEX_NORMALS
    out.world_normal = world_normal;
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        vertex.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

    return out;
}
