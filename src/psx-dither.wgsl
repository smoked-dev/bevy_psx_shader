#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings
#import bevy_sprite::{
    mesh2d_view_bindings::globals,
}

struct PsxDitherMaterial {
    replace_color: vec3<f32>,
    mult_color: vec3<f32>,
    dither_amount: f32,
    banding_enabled: u32,
    chroma_k: vec4<f32>,
};



@group(2) @binding(0)
var<uniform> material: PsxDitherMaterial;
@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;
@group(2) @binding(3)
var dither_color_texture: texture_2d<f32>;
@group(2) @binding(4)
var dither_color_sampler: sampler;
@group(2) @binding(5)
var lut_texture: texture_3d<f32>;
@group(2) @binding(6)
var lut_sampler: sampler;

fn random (noise: vec2<f32>) -> f32
{
    //--- Noise: Low Static (X axis) ---
    //return fract(sin(dot(noise.yx,vec2(0.000128,0.233)))*804818480.159265359);
    
    //--- Noise: Low Static (Y axis) ---
    //return fract(sin(dot(noise.xy,vec2(0.000128,0.233)))*804818480.159265359);
    
  	//--- Noise: Low Static Scanlines (X axis) ---
    //return fract(sin(dot(noise.xy,vec2(98.233,0.0001)))*925895933.14159265359);
    
   	//--- Noise: Low Static Scanlines (Y axis) ---
    //return fract(sin(dot(noise.xy,vec2(0.0001,98.233)))*925895933.14159265359);
    
    //--- Noise: High Static Scanlines (X axis) ---
    //return fract(sin(dot(noise.xy,vec2(0.0001,98.233)))*12073103.285);
    
    //--- Noise: High Static Scanlines (Y axis) ---
    //return fract(sin(dot(noise.xy,vec2(98.233,0.0001)))*12073103.285);
    
    //--- Noise: Full Static ---
    return fract(sin(dot(noise.xy,vec2(10.998,98.233)))*12433.14159265359);
}


fn channelError(col: f32, colMin: f32, colMax: f32) -> f32 {
	let range: f32 = abs(colMin - colMax);
	let aRange: f32 = abs(col - colMin);
	return aRange / range;
} 

fn ditheredChannel(error: f32, ditherBlockUV: vec2<f32>) -> f32 {
	let pattern: f32 = textureSample(dither_color_texture, dither_color_sampler, ditherBlockUV).r;
	if (error > pattern) {
		return 1.;
	} else { 
		return 0.;
	}
} 

fn ditheredChannel2(error: f32, ditherBlockUV: vec2<f32>, ditherSteps: f32) -> f32 {
    let error2 = floor(error * ditherSteps) / ditherSteps;
    var ditherUV = vec2(error2, 0.);
    ditherUV.x += ditherBlockUV.x;
    ditherUV.y = ditherBlockUV.y;
    return textureSample(dither_color_texture, dither_color_sampler, ditherUV).r; //tex2D(_DitherPattern, ditherUV).x;
}


fn RGBtoYUV(rgb: vec3<f32>) -> vec3<f32> {
	var yuv: vec3<f32>;
	yuv.r = rgb.r * 0.2126 + 0.7152 * rgb.g + 0.0722 * rgb.b;
	yuv.g = (rgb.b - yuv.r) / 1.8556 + 0.5;
	yuv.b = (rgb.r - yuv.r) / 1.5748 + 0.5;
/* 	var yuvgb = yuv.gb;
	yuvgb = yuv.gb + (0.5);
	yuv.g = yuvgb.x;
	yuv.b = yuvgb.y; */
	return yuv;
} 

fn YUVtoRGB(inyuv: vec3<f32>) -> vec3<f32> {
/* 	var yuvgb = yuv.gb;
	yuvgb = yuv.gb - (0.5);
	yuv.g = yuvgb.x;
	yuv.b = yuvgb.y; */
    var yuv = inyuv;
    yuv.g -= 0.5;
    yuv.b -= 0.5;
	return vec3<f32>(yuv.r * 1. + yuv.g * 0. + yuv.b * 1.5748, yuv.r * 1. + yuv.g * -0.187324 + yuv.b * -0.468124, yuv.r * 1. + yuv.g * 1.8556 + yuv.b * 0.);
} 

fn ditherColor(col: vec3<f32>, uv: vec2<f32>, xres: f32, yres: f32) -> vec3<f32> {
	var yuv: vec3<f32> = RGBtoYUV(col);
	let col1: vec3<f32> = floor(yuv * material.dither_amount) / material.dither_amount;
	let col2: vec3<f32> = ceil(yuv * material.dither_amount) / material.dither_amount;
//	let ditherBlockUV: vec2<f32> = uv * vec2<f32>(xres / 8., yres / 8.);

    
    let _MainTex_TexelSize = vec4(0.,0.,xres,yres);
    let _DitherPattern_TexelSize = vec4(0.,0.,36.,4.);
    let ditherSize = _DitherPattern_TexelSize.w;
    let ditherSteps = _DitherPattern_TexelSize.z/ditherSize;

    var ditherBlockUV: vec2<f32>  = uv;
    ditherBlockUV.x %= (ditherSize / _MainTex_TexelSize.z);
    ditherBlockUV.x /= (ditherSize / _MainTex_TexelSize.z);
    ditherBlockUV.y %= (ditherSize / _MainTex_TexelSize.w);
    ditherBlockUV.y /= (ditherSize / _MainTex_TexelSize.w);
    ditherBlockUV.x /= ditherSteps;

	yuv.x = mix(col1.x, col2.x, ditheredChannel2(channelError(yuv.x, col1.x, col2.x), ditherBlockUV, ditherSteps));
	yuv.y = mix(col1.y, col2.y, ditheredChannel2(channelError(yuv.y, col1.y, col2.y), ditherBlockUV, ditherSteps));
	yuv.z = mix(col1.z, col2.z, ditheredChannel2(channelError(yuv.z, col1.z, col2.z), ditherBlockUV, ditherSteps));
    
	return YUVtoRGB(yuv);
} 

fn ditherColor2(col: vec3<f32>, uv: vec2<f32>, xres: f32, yres: f32) -> vec3<f32> {
	var yuv: vec3<f32> = RGBtoYUV(col);
	let col1: vec3<f32> = floor(yuv * material.dither_amount) / material.dither_amount;
	let col2: vec3<f32> = ceil(yuv * material.dither_amount) / material.dither_amount;
	let ditherBlockUV: vec2<f32> = uv * vec2<f32>(xres / 8., yres / 8.);
	yuv.x = mix(col1.x, col2.x, ditheredChannel(channelError(yuv.x, col1.x, col2.x), ditherBlockUV));
	yuv.y = mix(col1.y, col2.y, ditheredChannel(channelError(yuv.y, col1.y, col2.y), ditherBlockUV));
	yuv.z = mix(col1.z, col2.z, ditheredChannel(channelError(yuv.z, col1.z, col2.z), ditherBlockUV));
	return YUVtoRGB(yuv);
} 




struct FragmentInput {
    @location(0) c_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct Wave {
    waves_x: f32,
    waves_y: f32,

    speed_x: f32,
    speed_y: f32,

    amplitude_x: f32,
    amplitude_y: f32
};

fn modulo(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
} 

fn pincush(uv: vec2<f32>, strength: f32) -> vec2<f32> {
    let st = uv - 0.5;
    let uvA = atan2(st.x, st.y);
    let uvD = dot(st, st);
    return 0.5 + (vec2(sin(uvA), cos(uvA)) * sqrt(uvD) * (1.0 - strength * uvD));
}



@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var uv_displaced = in.uv;

    let iResolution = vec2<f32>(textureDimensions(base_color_texture));

    let noise = (fract(sin(dot(in.uv * globals.time, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) * 2.0;
    //Noise stuff
    var maxStrength = 0.025;
    let minStrength = 0.125;

    let speed = 10.00;


  //  let uv = floor(uv_displaced.xy * iResolution) / iResolution;
   // let uv2 = fract(uv*fract(sin(globals.time*speed)));
    
    //--- Strength animate ---
//    maxStrength = clamp(sin(globals.time/2.0),minStrength,maxStrength);
    //-----------------------
    
    //--- Black and white ---


  //  var base_col = textureSample(base_color_texture, base_color_sampler, uv_displaced);

/* 
    let rChannel = textureSample(base_color_texture, base_color_sampler, pincush(uv_displaced, 0.3 * 0.3)).r;
    let gChannel = textureSample(base_color_texture, base_color_sampler, pincush(uv_displaced, 0.15 * 0.3)).g;
    let bChannel = textureSample(base_color_texture, base_color_sampler, pincush(uv_displaced, 0.075 * 0.3)).b;
    var base_col = vec4(rChannel, gChannel, bChannel, 1.);
 */
    let pixel_size_x = 1.0 / iResolution.x;
    let pixel_size_y = 1.0 / iResolution.y;
    var base_col = textureSample(base_color_texture, base_color_sampler, uv_displaced);
    let color_left = textureSample(base_color_texture, base_color_sampler, uv_displaced - vec2(pixel_size_x, pixel_size_y));

    // Aberration is applied only to the *difference* between the taps, so flat colors stay flat.
    let diff = base_col.rgb - color_left.rgb;
    base_col = vec4<f32>(base_col.rgb + diff * material.chroma_k.rgb, base_col.a);
//    base_col = vec4(base_col.rgb + dpdx(base_col.rgb)*vec3(3.,0.,-3.), base_col.a);
    base_col = base_col;

//    base_col = vec4(base_col.rgb + dpdx(base_col.rgb)*vec3(3.,0.,-3.), base_col.a);

    base_col = base_col * vec4<f32>(material.mult_color, 1.);

/*     let chroma = dpdx(base_col.rgb)*vec3(3.,0.,-3.);
    base_col = textureSample(base_color_texture, base_color_sampler, uv_displaced);

    base_col.r += chroma.r;
    base_col.g += chroma.g;
    base_col.b += chroma.b;
 */
    var final_col = ditherColor(base_col.rgb, uv_displaced, iResolution.x * 1., iResolution.y * 1.);

    let half_texel = vec3<f32>(1.0 / 64. / 2.);

    // Notice the ".rbg".
    // If we sample the LUT using ".rgb" instead,
    // the way the 3D texture is loaded will mean the
    // green and blue colors are swapped.
    // This mitigates that.


    let raw_color = final_col.rbg;// - colour * 0.5;
    final_col = vec4<f32>(textureSample(lut_texture, lut_sampler, raw_color + half_texel).rgb, 1.0).rgb;
//    final_col += vec3(noise * 0.035);
//    final_col *= 1.0 - 0.12 * step(0.5, fract(floor(uv_displaced.y * iResolution.y) * 0.5)); // scanlines (comment out to disable)
    return vec4(final_col, 1.0);
}
