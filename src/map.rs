use crate::{
    asset_loader::{Sprites, SpritesLoadingStates},
    physics::{MovingObject, MovingSpriteSheetBundle, Position, AABB},
};
use bevy::prelude::*;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapAabb>()
            .add_systems(OnEnter(SpritesLoadingStates::Finished), setup_map);
    }
}

#[derive(Resource)]
pub struct MapAabb {
    pub size: AABB,
}
impl Default for MapAabb {
    fn default() -> Self {
        Self {
            size: AABB::new(Vec2::splat(100.0)),
        }
    }
}

pub const TILE_SIZE: f32 = 64.0;

pub fn setup_map(mut commands: Commands, sprites: Res<Sprites>, images: Res<Assets<Image>>) {
    // loading image and getting image size
    let level1_image = images.get(&sprites.level1).unwrap();
    let size = level1_image.size();

    commands.insert_resource(MapAabb {
        size: AABB::new(size.as_vec2() * TILE_SIZE / 2.0),
    });

    let mut blocks: Vec<(UVec2, UVec2)> = Vec::new();
    // iterating over every pixel
    for y in 0..size.x {
        for x in 0..size.y {
            let pixel_index = (y * level1_image.size().y + x) as usize * 4; // Assuming 4 bytes per pixel (RGBA)
            let rgba = &level1_image.data[pixel_index..pixel_index + 4];

            match rgba {
                [255, 255, 255, 255] => {
                    let mut added = false;
                    for block in &mut blocks {
                        // Vertical:
                        // if the new block is in the same horizontal line and one below an existing block,
                        // add it to the existing block
                        if block.0.x == x && block.1.x == x && y == block.1.y + 1 {
                            block.1.y += 1;

                            added = true;
                            break;
                        }
                        // Horizontal:
                        // if the new block is in the same vertical line and one to the right of an existing block,
                        // add it to the existing block
                        if block.0.y == y && block.1.y == y && x == block.1.x + 1 {
                            block.1.x += 1;

                            added = true;
                            break;
                        }
                    }
                    // if the new block wasnt added to any existing ones, add it to the vec
                    if !added {
                        blocks.push((UVec2::new(x, y), UVec2::new(x, y)));
                    }
                }
                _ => {}
            }
        }
    }

    for block in blocks {
        let dimensions = Vec2::new(
            (block.1.x - block.0.x) as f32,
            (block.1.y - block.0.y) as f32,
        );

        let mut halfsize = dimensions / 2.0;

        let original_position = block.0.as_vec2() + halfsize;

        // convert to bevy coordinates
        let mut position = Vec2::new(
            original_position.x - size.as_vec2().x / 2.0,
            size.as_vec2().y / 2.0 - original_position.y,
        );
        // scaling the values up
        halfsize *= TILE_SIZE;
        halfsize += TILE_SIZE / 2.0;
        position *= TILE_SIZE;

        commands.spawn((
            Name::new("Block"),
            MovingSpriteSheetBundle {
                spritesheet_bundle: SpriteSheetBundle {
                    atlas: TextureAtlas {
                        layout: sprites.map_layout.clone(),
                        index: 0,
                    },
                    texture: sprites.map_texture.clone(),
                    sprite: Sprite {
                        custom_size: Some(halfsize * 2.0),
                        ..default()
                    },
                    ..default()
                },
                aabb: AABB::new(halfsize),
                moving_object: MovingObject {
                    position: Position::new(position),
                    ..default()
                },
                ..default()
            },
        ));
    }
}
