use crate::{
    asset_loader::{load_assets, Sprites},
    physics::{Position, AABB},
};
use bevy::prelude::*;
use image::io::Reader as ImageReader;
use image::GenericImageView;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_map.after(load_assets));
    }
}

pub const TILE_SIZE: f32 = 64.0;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TileType {
    Empty,
    Block,
}

#[derive(Resource)]
pub struct TileMap {
    tile_map: Box<[TileType]>,
    width: usize,
    height: usize,
}

impl TileMap {
    // Initialize with a specific width and height
    fn new(width: usize, height: usize) -> Self {
        let tile_map = vec![TileType::Empty; width * height].into_boxed_slice();
        Self {
            tile_map,
            width,
            height,
        }
    }
}

impl TileMap {
    pub fn world_coordinates_to_map_index(&self, coordinates: Vec2) -> (usize, usize) {
        (
            (self.height as f32 / 2.0 + coordinates.x / TILE_SIZE).round() as usize,
            (self.width as f32 / 2.0 - coordinates.y / TILE_SIZE).round() as usize,
        )
    }
    pub fn map_index_to_world_coordinates(&self, index: (usize, usize)) -> Vec2 {
        Vec2::new(
            (index.0 as f32 - self.height as f32 / 2.0) * TILE_SIZE,
            (self.width as f32 / 2.0 - index.1 as f32) * TILE_SIZE,
        )
    }
    // Accessor method to work with the flat array as if it were 2D
    fn get_tile(&self, index: (usize, usize)) -> Option<&TileType> {
        let x = index.0;
        let y = index.1;
        self.tile_map.get(y * self.width + x)
    }
    fn get_mut_tile(&mut self, index: (usize, usize)) -> Option<&mut TileType> {
        let x = index.0;
        let y = index.1;
        self.tile_map.get_mut(y * self.width + x)
    }
    pub fn is_obstacle(&self, index: (usize, usize)) -> bool {
        self.get_tile(index)
            .map_or(true, |tile_type| *tile_type == TileType::Block)
    }
    pub fn is_tile(&self, index: (usize, usize), tile: TileType) -> bool {
        self.get_tile(index) == Some(&tile)
    }
}

pub fn has_ground(aabb: &AABB, position: Position, map: &TileMap) -> Option<f32> {
    // boundaries of the bottom sensor
    let bottom_left = position.value - aabb.halfsize + Vec2::new(1.0, -1.0);
    let bottom_right = Vec2::new(bottom_left.x + aabb.halfsize.x * 2.0 - 2.0, bottom_left.y);

    // check all of the tiles that lie along the bottom sensor
    let mut checked_tile = bottom_left;
    loop {
        // only check up to the right side (necessary because of tilesize incrementation)
        checked_tile.x = f32::min(checked_tile.x, bottom_right.x);
        let tile_index = map.world_coordinates_to_map_index(checked_tile);

        let ground_y = map.map_index_to_world_coordinates(tile_index).y + TILE_SIZE / 2.0;

        if map.is_obstacle(tile_index) {
            return Some(ground_y);
        }

        if checked_tile.x >= bottom_right.x {
            break;
        }

        checked_tile.x += TILE_SIZE;
    }
    None
}

pub fn collides_with_ceiling(aabb: &AABB, position: Position, map: &TileMap) -> Option<f32> {
    // boundaries of the top sensor
    let top_right = position.value + aabb.halfsize + Vec2::new(-1.0, 1.0);
    let top_left = Vec2::new(top_right.x - aabb.halfsize.x * 2.0 + 2.0, top_right.y);

    // check all of the tiles that lie along the top sensor
    let mut checked_tile = top_left;
    loop {
        // only check up to the right side (necessary because of tilesize incrementation)
        checked_tile.x = f32::min(checked_tile.x, top_right.x);
        let tile_index = map.world_coordinates_to_map_index(checked_tile);

        if map.is_obstacle(tile_index) {
            return Some(map.map_index_to_world_coordinates(tile_index).y - TILE_SIZE / 2.0);
        }

        if checked_tile.x >= top_right.x {
            break;
        }

        checked_tile.x += TILE_SIZE;
    }
    None
}

pub fn collides_with_left_wall(aabb: &AABB, position: Position, map: &TileMap) -> Option<f32> {
    // boundaries of the left sensor
    let bottom_left = position.value - aabb.halfsize + Vec2::new(-1.0, 1.0);
    let top_left = Vec2::new(bottom_left.x, bottom_left.y + aabb.halfsize.y * 2.0 - 2.0);

    // check all of the tiles that lie along the left sensor
    let mut checked_tile = bottom_left;
    loop {
        // only check up to the right side (necessary because of tilesize incrementation)
        checked_tile.y = f32::min(checked_tile.y, top_left.y);
        let tile_index = map.world_coordinates_to_map_index(checked_tile);

        if map.is_obstacle(tile_index) {
            return Some(map.map_index_to_world_coordinates(tile_index).x + TILE_SIZE / 2.0);
        }

        if checked_tile.y >= top_left.y {
            break;
        }

        checked_tile.y += TILE_SIZE;
    }
    None
}

pub fn collides_with_right_wall(aabb: &AABB, position: Position, map: &TileMap) -> Option<f32> {
    // boundaries of the left sensor
    let top_right = position.value + aabb.halfsize + Vec2::new(1.0, -1.0);
    let bottom_right = Vec2::new(top_right.x, top_right.y - aabb.halfsize.y * 2.0 + 2.0);

    // check all of the tiles that lie along the left sensor
    let mut checked_tile = bottom_right;
    loop {
        // only check up to the right side (necessary because of tilesize incrementation)
        checked_tile.y = f32::min(checked_tile.y, top_right.y);
        let tile_index = map.world_coordinates_to_map_index(checked_tile);

        if map.is_obstacle(tile_index) {
            return Some(map.map_index_to_world_coordinates(tile_index).x - TILE_SIZE / 2.0);
        }

        if checked_tile.y >= top_right.y {
            break;
        }

        checked_tile.y += TILE_SIZE;
    }
    None
}

fn setup_map(mut commands: Commands, texture_atlas_handle: Res<Sprites>) {
    // load image
    let img = ImageReader::open("assets/map1.png")
        .unwrap()
        .decode()
        .unwrap();

    // create empty tileMap
    let dimensions = img.dimensions();
    let mut tile_map = TileMap::new(dimensions.0 as usize, dimensions.1 as usize);

    // populate tilemap
    for (x, y, pixel) in img.pixels() {
        let rgba = pixel.0;

        match rgba {
            [255, 255, 255, 255] => {
                // index should be valid, as width and height are constructed from the same
                // dimensions
                let tile = tile_map.get_mut_tile((x as usize, y as usize)).unwrap();
                *tile = TileType::Block;
            }
            _ => {}
        }
    }

    for i in 0..tile_map.tile_map.len() {
        // Calculate the x-coordinate
        let x = i % tile_map.width;
        // Calculate the y-coordinate
        let y = i / tile_map.width;
        let tile_type = &tile_map.tile_map[i]; // Access the TileType directly by index

        match tile_type {
            TileType::Empty => {}
            TileType::Block => {
                let coordinates = tile_map.map_index_to_world_coordinates((x, y));
                commands.spawn((
                    Name::new("Block"),
                    SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle.map_atlas.clone(),
                        sprite: TextureAtlasSprite {
                            index: 0,
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(coordinates.x, coordinates.y, 0.0),
                        ..default()
                    },
                    AABB::new(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0)),
                ));
            }
        }
    }

    commands.insert_resource(tile_map);
}
