use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub fn init(app: &mut App) {
    app.add_plugins(MaterialPlugin::<GridMaterial>::default())
        .init_asset::<GridMaterial>();
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub thickness: f32,
}

impl Material for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
