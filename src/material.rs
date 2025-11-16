use bevy::{prelude::*, reflect::TypePath, render::render_resource::*, sprite::Material2d};

pub const PSX_FRAG_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(310591614790536);
pub const PSX_DITH_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(210541614790536);
pub const PSX_DITHER_HANDLE: Handle<Image> = Handle::weak_from_u128(510291613494514);
pub const PSX_VERT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(120592519790135);
pub const PSX_LUT_HANDLE: Handle<Image> = Handle::weak_from_u128(120592519790132);

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
