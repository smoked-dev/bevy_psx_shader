use bevy::{asset::uuid_handle, prelude::*, reflect::TypePath, render::render_resource::*, shader::ShaderRef, sprite_render::Material2d};

pub const PSX_FRAG_SHADER_HANDLE: Handle<Shader> = uuid_handle!("a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d");
pub const PSX_DITH_SHADER_HANDLE: Handle<Shader> = uuid_handle!("b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e");
pub const PSX_DITHER_HANDLE: Handle<Image> = uuid_handle!("c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f");
pub const PSX_VERT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("d4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f8a");
pub const PSX_LUT_HANDLE: Handle<Image> = uuid_handle!("e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8a9b");

impl Material for PsxMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PSX_FRAG_SHADER_HANDLE)
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(PSX_VERT_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

impl Material2d for PsxDitherMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PSX_DITH_SHADER_HANDLE)
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct PsxMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(0)]
    pub fog_color: LinearRgba,
    #[uniform(0)]
    pub snap_amount: f32,
    #[uniform(0)]
    pub fog_distance: Vec2,
    // #[uniform(0)]
    // pub dither_amount: f32,
    // #[uniform(0)]
    // pub banding_enabled: u32,
    /// First one is start second is end
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    // #[texture(3)]
    // #[sampler(4, sampler_type = "non_filtering")]
    // pub dither_color_texture: Option<Handle<Image>>,
}

impl Default for PsxMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::WHITE,
            fog_color: LinearRgba::WHITE,
            snap_amount: 5.0,
            fog_distance: Vec2::new(25.0, 75.0),
            // dither_amount: 64.0,
            color_texture: None,
            alpha_mode: AlphaMode::Opaque,
            // dither_color_texture: Some(PSX_DITHER_HANDLE.typed()),
            // banding_enabled: 0,
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct PsxDitherMaterial {
    #[uniform(0)]
    pub replace_color: Vec3,
    #[uniform(0)]
    pub mult_color: Vec3,
    #[uniform(0)]
    pub dither_amount: f32,
    #[uniform(0)]
    pub banding_enabled: u32,
    /// Chromatic aberration controls (x/y/z = RGB), applied to the difference between taps.
    ///
    /// `0.0` disables the effect for that channel.
    #[uniform(0)]
    pub chroma_k: Vec4,

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    /// First one is start second is end
    #[texture(3)]
    // #[sampler(4, sampler_type = "non_filtering")]
    #[sampler(4)]
    pub dither_color_texture: Option<Handle<Image>>,

    #[texture(5, dimension = "3d")]
    #[sampler(6)]
    pub lut_texture: Option<Handle<Image>>,
}

impl Default for PsxDitherMaterial {
    fn default() -> Self {
        Self {
            replace_color: Vec3::new(0., 0., 0.),
            mult_color: Vec3::ONE,
            dither_amount: 8.0,
            dither_color_texture: Some(PSX_DITHER_HANDLE),
            banding_enabled: 1,
            // Matches the previous hard-coded weights, but parameterized as:
            // out = current + k * (current - left)
            chroma_k: Vec4::new(0.2, -0.5, -1.2, 0.0),
            color_texture: None,
            lut_texture: Some(PSX_LUT_HANDLE),
        }
    }
}

/// A look-up texture. Maps colors to colors. Useful for colorschemes.
#[derive(Debug, Component, Clone)]
pub struct Lut {
    /// The 3D look-up texture
    texture: Handle<Image>,

    prepared: bool,
}

impl Lut {
    /// Creates a new LUT component.
    /// The image should be a 64x64x64 3D texture.
    /// See the `make-neutral-lut` example.
    pub fn new(texture: Handle<Image>) -> Self {
        Self {
            texture,
            prepared: false,
        }
    }
}

pub fn adapt_image_for_lut_use(
    mut assets: ResMut<Assets<Image>>,
    mut luts: Query<&mut Lut, Changed<Lut>>,
) {
    for mut lut in luts.iter_mut() {
        if lut.prepared {
            continue;
        }

        let image = assets
            .get_mut(&lut.texture)
            .expect("Handle should point to asset");

        // The LUT is a 3d texture. It has 64 layers, each of which is a 64x64 image.
        image.texture_descriptor.size = Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 64,
        };
        image.texture_descriptor.dimension = TextureDimension::D3;
        image.texture_descriptor.format = TextureFormat::Rgba8Unorm;

        image.texture_view_descriptor = Some(TextureViewDescriptor {
            label: Some("LUT Texture View"),
            format: Some(TextureFormat::Rgba8Unorm),
            dimension: Some(TextureViewDimension::D3),
            ..default()
        });

        debug!("LUT prepared for handle {:?}", lut.texture);
        lut.prepared = true;
    }
}
