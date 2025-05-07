use std::path::Path;

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, LoadContext},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::player::Info;

#[derive(Asset, TypePath, Default, Deserialize, Serialize)]
pub struct Data {
    pub player_info: Info,
    pub player_position: Vec3,
}

impl AssetLoader for Data {
    type Asset = Data;
    type Settings = ();
    type Error = ron::de::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let save_data = ron::de::from_bytes::<Data>(&bytes)?;
        Ok(save_data)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource)]
pub struct ResHandle(pub Handle<Data>);

pub fn load<'a>(
    mut commands: Commands,
    asset_server: &ResMut<AssetServer>,
    path: impl Into<AssetPath<'a>>,
) -> Handle<Data> {
    info!("Load");
    let saved_data = asset_server.load::<Data>(path);
    commands.insert_resource(ResHandle(saved_data.clone()));
    saved_data
}

pub fn save(
    data_assets: ResMut<Assets<Data>>,
    res_handle: Res<ResHandle>,
    file_path: impl AsRef<Path>,
) {
    // info!("Save");
    let data = data_assets.get(&res_handle.0).unwrap();
    let bytes = ron::ser::to_string(data).unwrap().into_bytes();
    std::fs::write(file_path, bytes).unwrap();
}
