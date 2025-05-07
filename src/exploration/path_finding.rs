use std::{cell::RefCell, rc::Rc};

use crate::player::{CameraPivot, MainPlayer};
use crate::{map, player};
use bevy::{
    prelude::*,
    tasks::{block_on, poll_once, AsyncComputeTaskPool, Task},
};

use super::ChunkChange;

#[derive(Resource)]
pub struct Path(Vec<map::TileCoordinate>);

#[derive(Component)]
pub struct PathFindingTask(Task<Path>);

pub fn poll_path_finding_tasks(
    mut commands: Commands,
    mut q_task: Query<(Entity, &mut PathFindingTask)>,
    mut q_player: Query<&mut Transform, (With<MainPlayer>, Without<CameraPivot>)>,
    q_camera: Query<&mut Transform, (With<CameraPivot>, Without<MainPlayer>)>,
) {
    for (task_entity, mut task) in q_task.iter_mut() {
        if let Some(content) = block_on(poll_once(&mut task.0)) {
            commands.entity(task_entity).despawn();
            let end = content.0.last().unwrap();
            // Move player
            let new_location = end.to_world() + Vec3::new(0.0, player::PLAYER_SIZES.y / 2.0, 0.0);
            q_player.single_mut().translation = new_location;
            // Move camera
            let chunk_coord = end.to_chunk();
            let new_camera_position = chunk_coord.world_center();
            let camera = q_camera.single();
            if (new_camera_position - camera.translation).length() > 1.0 {
                commands.trigger(ChunkChange(chunk_coord));
            }
            commands.insert_resource(content);
        }
        break;
    }
}

#[derive(Debug)]
struct Node {
    coordinate: map::TileCoordinate,
    parent: Option<Rc<RefCell<Box<Node>>>>,
    g: f32,
    h: f32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.coordinate == other.coordinate
    }
}

impl Node {
    pub fn total(&self) -> f32 {
        self.g + self.h
    }
}

fn distance(a: map::TileCoordinate, b: map::TileCoordinate) -> f32 {
    let vector = a - b;
    (vector.x.abs() + vector.z.abs()) as f32
}

pub fn spawn_path_finding_task(
    mut commands: Commands,
    start: map::TileCoordinate,
    end: map::TileCoordinate,
    chunk_assets: Res<Assets<map::Chunk>>,
    current_chunk: Res<map::CurrentChunk>,
) {
    let chunk_data = map::Chunk(
        chunk_assets
            .get(current_chunk.grid.id())
            .expect("Requested path to be in a loaded chunk")
            .0,
    );
    let task_handle = AsyncComputeTaskPool::get().spawn(async move {
        let start_node = Node {
            coordinate: start,
            parent: None,
            g: 0.0,
            h: distance(start, end),
        };
        let mut open = vec![Rc::new(RefCell::new(Box::new(start_node)))];
        let mut close = Vec::<Rc<RefCell<Box<Node>>>>::new();

        while !open.is_empty() {
            let (current_node, current_node_index) = {
                let mut lowest_index = 0;
                for (index, node) in open.iter().enumerate() {
                    if (**node).borrow().total() < (*open[lowest_index]).borrow().total() {
                        lowest_index = index;
                    }
                }
                (open[lowest_index].clone(), lowest_index)
            };

            if (*current_node).borrow_mut().coordinate == end {
                let mut path = Vec::<map::TileCoordinate>::new();
                let mut current = current_node.clone();
                loop {
                    if (*current).borrow().parent.is_none() {
                        break;
                    }
                    path.push((*current).borrow_mut().coordinate);
                    let parent: Rc<RefCell<Box<Node>>> =
                        (*current).borrow_mut().parent.take().unwrap();
                    current = parent.clone();
                }
                path.push(start);
                path.reverse();
                return Path(path);
            }
            close.push(current_node.clone());
            open.remove(current_node_index);

            let mut neighbors = Vec::<Node>::new();
            let origin = current_node.as_ref().borrow().coordinate;
            for x in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && z == 0 {
                        continue;
                    }
                    let neighbor = origin + map::TileCoordinate::new(x, z);
                    let neighbor_local = neighbor.to_local();
                    if let Some(tile) = chunk_data.get_tile(neighbor_local) {
                        if *tile == map::Tile::Ground {
                            neighbors.push(Node {
                                coordinate: neighbor,
                                parent: Some(current_node.clone()),
                                g: (*current_node).borrow().g
                                    + distance((*current_node).borrow().coordinate, neighbor),
                                h: distance(neighbor, end),
                            });
                        }
                    }
                }
            }

            for neighbor in neighbors {
                if close
                    .iter()
                    .find(|node: &&Rc<RefCell<Box<Node>>>| {
                        (***node).borrow().coordinate == neighbor.coordinate // && node.g < neighbor.g
                    })
                    .is_some()
                {
                    continue;
                }
                if let Some(index) = open
                    .iter()
                    .position(|node| (**node).borrow().coordinate == neighbor.coordinate)
                {
                    if neighbor.g < (*open[index]).borrow().g {
                        (*open[index]).borrow_mut().parent = Some(open[index].clone());
                        (*open[index]).borrow_mut().g = neighbor.g;
                    }
                } else {
                    open.push(Rc::new(RefCell::new(Box::new(neighbor))));
                }
            }
        }

        Path(vec![start, end])
    });
    commands.spawn_empty().insert(PathFindingTask(task_handle));
}

#[cfg(debug_assertions)]
pub fn draw_path_finding(
    path_res: Option<Res<Path>>,
    mut gizmos: Gizmos,
    // q_player: Query<&Transform, With<MainPlayer>>,
) {
    let Some(path) = path_res else {
        return;
    };
    // let chunk_coord = map::ChunkCoordinate::from_world(q_player.single().translation);
    let mut index = 0;
    while index < path.0.len() - 1 {
        let start = path.0[index];
        let end = path.0[index + 1];
        gizmos.line(start.to_world(), end.to_world(), Color::BLACK);
        index += 1;
    }
}
