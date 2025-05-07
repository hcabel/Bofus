use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use serde::{Deserialize, Serialize};

use crate::map;

use super::{Chunk, SIZE_X, SIZE_Z};

#[derive(Default)]
pub struct Loader;

impl AssetLoader for Loader {
    type Asset = Chunk;
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
        let map_asset = ron::de::from_bytes::<Chunk>(&bytes)?;
        Ok(map_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl Serialize for Chunk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct FileStructure<'a>(Vec<Vec<&'a map::Tile>>);

        let tiles = FileStructure(self.0.iter().map(|row| row.iter().collect()).collect());
        tiles.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Chunk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FileStructure(Vec<Vec<map::Tile>>);

        let mut chunk = Chunk([[map::Tile::default(); SIZE_X]; SIZE_Z]);
        let data = FileStructure::deserialize(deserializer)?;
        for (z, row) in data.0.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                chunk.0[z][x] = *tile;
            }
        }
        Ok(chunk)
    }
}
