use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub fn init(app: &mut App) {
    app.add_plugins(MaterialPlugin::<PlayerShadowMaterial>::default())
        .init_asset::<PlayerShadowMaterial>();
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlayerShadowMaterial {}

impl Material for PlayerShadowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/player_shadow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
