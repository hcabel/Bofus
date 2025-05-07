use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use serde::{Deserialize, Serialize};

use crate::map::{self, Map};

#[derive(Default)]
pub struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = map::Map;
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
        let map_asset = ron::de::from_bytes::<map::Map>(&bytes)?;
        Ok(map_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ChunkData<'a> {
            x: usize,
            z: usize,
            tiles: Vec<Vec<&'a map::Tile>>,
        }

        let mut chunks = Vec::new();
        for chunk_z in 0..map::SIZE_Z {
            for chunk_x in 0..map::SIZE_X {
                let chunk = &self.0[chunk_z][chunk_x];
                let chunk_data = ChunkData {
                    x: chunk_x,
                    z: chunk_z,
                    tiles: chunk.0.iter().map(|row| row.iter().collect()).collect(),
                };
                chunks.push(chunk_data);
            }
        }
        chunks.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Map {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ChunkData {
            x: usize,
            z: usize,
            tiles: Vec<Vec<map::Tile>>,
        }

        let chunks: Vec<ChunkData> = Deserialize::deserialize(deserializer)?;
        let mut map = Map::default();
        for chunk_data in chunks {
            let chunk = &mut map.0[chunk_data.z][chunk_data.x];
            for (z, row) in chunk_data.tiles.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    chunk.0[z][x] = *tile;
                }
            }
        }
        Ok(map)
    }
}
