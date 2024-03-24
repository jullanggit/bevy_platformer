use bevy::{prelude::*, window::PrimaryWindow};
use rand::{thread_rng, Rng};

use crate::{
    asset_loader::SpritesLoadingStates,
    map::{setup_map, MapAabb, TileType},
    physics::{MovingObject, Position, Velocity, AABB},
    player::Player,
    quadtree::build_quadtree,
};

pub struct BoidPlugin;
impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidParameters>()
            .init_resource::<BoidParameters>()
            .add_systems(
                OnEnter(SpritesLoadingStates::Finished),
                spawn_boids.after(setup_map),
            )
            .add_systems(Update, move_boids);
    }
}

#[derive(Component, Debug, Default, Clone)]
struct Boid {
    inside_target: bool,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct BoidParameters {
    max_velocity: f32,
    min_velocity: f32,
    view_distance: f32,
    view_distance_aabb: AABB,

    avoid_factor: f32,
    centering_factor: f32,
    matching_factor: f32,
    multiplier: f32,
    random_factor: f32,

    pub disperse: bool,
    disperse_factor: f32,

    avoid_obstacles_factor: f32,
    avoid_obstacles_offset: f32,

    attract_target_factor: f32,
    attract_target_offset: f32,

    edge_avoidance_distance: f32,
    edge_avoidance_strength: f32,

    quadtree_capacity: usize,

    player_push_factor: f32,
}
impl Default for BoidParameters {
    fn default() -> Self {
        let view_distance = 25.0;
        Self {
            max_velocity: 600.0,
            min_velocity: 40.0,
            view_distance,
            view_distance_aabb: AABB::new(Vec2::splat(view_distance)),

            avoid_factor: 3.0,
            centering_factor: 0.005,
            matching_factor: 0.1,
            multiplier: 2.0,
            random_factor: 0.1,

            disperse: false,
            disperse_factor: 20.0,

            avoid_obstacles_factor: 200.0,
            avoid_obstacles_offset: 10.0,

            attract_target_factor: 200.0,
            attract_target_offset: 10.0,

            edge_avoidance_distance: 10.0,
            edge_avoidance_strength: 10.0,

            quadtree_capacity: 2,

            player_push_factor: 0.1,
        }
    }
}

fn move_boids(
    mut query: Query<(
        Option<&AABB>,
        &mut MovingObject,
        Entity,
        Option<&Boid>,
        Option<&mut TileType>,
    )>,
    map_aabb: Res<MapAabb>,
    boid_params: Res<BoidParameters>,
    window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let window = window.get_single().expect("No Primary window");
    let window_halfsize = 0.5 * Vec2::new(window.width(), window.height());

    // new
    let quadtree = build_quadtree(
        &query,
        &AABB::new(window_halfsize),
        boid_params.quadtree_capacity,
        |(aabb, moving_object, entity, _, _)| (aabb, moving_object, entity),
    );

    let mut boids = Vec::new();

    // collect all boids and the stuff in their view range
    for (aabb, moving_object, entity, _, _) in &query {
        // Filter out non-boids
        if aabb.is_some() {
            continue;
        }

        let position = moving_object.position;

        let mut other_stuff = Vec::new();
        quadtree.query(&boid_params.view_distance_aabb, position, &mut other_stuff);

        boids.push((entity, other_stuff));
    }

    let mut rng = thread_rng();

    let mut to_teleport = Vec::new();

    // iterate over all boids and the boids in their view range
    for (a_entity, others) in boids {
        let mut final_velocity = Vec2::ZERO;

        // Calculate total_position, total_velocity and how much should be steered away from other
        // boids
        let (total_position, total_velocity, boids_amount) = others.iter().fold(
            (Vec2::ZERO, Vec2::ZERO, 0.0),
            |(pos_acc, vel_acc, amount_acc), b_entity| {
                // just return the accumulators if a and b are the same entity, essentialy skipping the iteration
                if a_entity == *b_entity {
                    return (pos_acc, vel_acc, amount_acc);
                }

                // get components of both entities
                let [(_, mut a_moving_object, _, boid, _), (b_aabb, mut b_moving_object, _, _, tile_type)] =
                    query.get_many_mut([a_entity, *b_entity]).unwrap();

                // If b_entity should be teleportet, and the current boid already has some boids
                // arond it
                if to_teleport.contains(b_entity) && amount_acc >= 7.0 {
                    b_moving_object.position.value = a_moving_object.position.value;
                }

                // define values for easier access
                let a_position = a_moving_object.position.value;
                let a_velocity = a_moving_object.velocity.value;
                let b_position = b_moving_object.position.value;
                let b_velocity = b_moving_object.velocity.value;

                object_interactions(b_aabb, a_position, b_position, &boid_params, &mut final_velocity, a_velocity, a_moving_object, &time, &mut rng, &map_aabb, tile_type);

                // add to the accumulator
                (pos_acc + b_position, vel_acc + b_velocity, amount_acc + 1.0)
            },
        );
        // If there arent any boids around the current boid
        if boids_amount < 3.0 {
            to_teleport.push(a_entity);
        }

        // Get components of a_entity again, might be able to optimize
        let (_, mut a_moving_object, _, _, _) = query.get_mut(a_entity).unwrap();
        let a_position = a_moving_object.position.value;
        let a_velocity = a_moving_object.velocity.value;

        // Steer away from edges of the window
        if a_position.x < -window_halfsize.x + boid_params.edge_avoidance_distance {
            final_velocity.x += boid_params.edge_avoidance_strength;
        } else if a_position.x > window_halfsize.x - boid_params.edge_avoidance_distance {
            final_velocity.x -= boid_params.edge_avoidance_strength;
        }
        if a_position.y < -window_halfsize.y + boid_params.edge_avoidance_distance {
            final_velocity.y += boid_params.edge_avoidance_strength;
        } else if a_position.y > window_halfsize.y - boid_params.edge_avoidance_distance {
            final_velocity.y -= boid_params.edge_avoidance_strength;
        }

        // steer towards percieved center
        if boids_amount > 0.0 {
            let percieved_center = (total_position - a_position) / boids_amount;

            if boid_params.disperse {
                final_velocity += (a_position - percieved_center)
                    * boid_params.centering_factor
                    * boid_params.disperse_factor;
            } else {
                final_velocity += (percieved_center - a_position) * boid_params.centering_factor;
            }
        }

        // steer in the same direction as the other boids
        if boids_amount > 0.0 {
            let percieved_velocity = ((total_velocity - a_velocity) / boids_amount).normalize()
                * boid_params.max_velocity;
            final_velocity += (percieved_velocity - a_velocity) * boid_params.matching_factor;
        }

        // Normalize velocity
        let final_velocity_length = final_velocity.length();
        if final_velocity_length > 0.0 {
            if final_velocity_length > boid_params.max_velocity {
                final_velocity = final_velocity.normalize() * boid_params.max_velocity;
            } else if final_velocity_length < boid_params.min_velocity {
                final_velocity = final_velocity.normalize() * boid_params.min_velocity;
            }
        }
        // random movement
        final_velocity.x +=
            (rng.gen_range(-0.3..0.3)) * boid_params.max_velocity * boid_params.random_factor;
        final_velocity.y +=
            (rng.gen_range(-0.3..0.3)) * boid_params.max_velocity * boid_params.random_factor;

        a_moving_object.velocity.value += final_velocity * boid_params.multiplier;
    }
}

fn object_interactions(
    b_aabb: Option<&AABB>,
    a_position: Vec2,
    b_position: Vec2,
    boid_params: &Res<'_, BoidParameters>,
    final_velocity: &mut Vec2,
    a_velocity: Vec2,
    mut a_moving_object: Mut<'_, MovingObject>,
    time: &Res<'_, Time>,
    rng: &mut rand::prelude::ThreadRng,
    map_aabb: &Res<'_, MapAabb>,
    tile_type: Option<Mut<'_, TileType>>,
) {
    match tile_type {
        Some(tile_type) => {
            match tile_type.into_inner() {
                TileType::Tile => {
                    let b_aabb = b_aabb.expect("Tile doesnt have aabb");
                    let closest_point = a_position
                        .clamp(b_position - b_aabb.halfsize, b_position + b_aabb.halfsize);

                    let distance_to_closest_point = a_position.distance(closest_point);

                    // If the boid isn't inside the block
                    if distance_to_closest_point > 0.0 {
                        let avoid_strength =
                            boid_params.avoid_obstacles_factor / distance_to_closest_point;
                        let avoid_vec = (a_position - closest_point).normalize() * avoid_strength;
                        *final_velocity += avoid_vec;
                    // If the boid is inside the block, and has a large enoug velocity, push it
                    // backwards
                    } else if a_velocity.length() > 5.0 {
                        a_moving_object.position.value -= a_velocity * time.delta_seconds();
                    } else {
                        // Teleport the boid to a random location
                        a_moving_object.position.value = Vec2::new(
                            rng.gen_range(-map_aabb.size.halfsize.x..map_aabb.size.halfsize.x),
                            rng.gen_range(-map_aabb.size.halfsize.y..map_aabb.size.halfsize.y)
                                / 2.0,
                        );
                        // Set random velocity
                        a_moving_object.velocity.value.x =
                            (rng.gen::<f32>() - 0.5) * 2.0 * boid_params.max_velocity;
                        a_moving_object.velocity.value.y =
                            (rng.gen::<f32>() - 0.5) * 2.0 * boid_params.max_velocity;
                    }
                }
                TileType::Target(mut hp) => {
                    let b_aabb = b_aabb.expect("Tile doesnt have aabb");
                    let closest_point = a_position
                        .clamp(b_position - b_aabb.halfsize, b_position + b_aabb.halfsize);

                    let distance_to_closest_point = a_position.distance(closest_point);

                    // If the boid isn't inside the target
                    if distance_to_closest_point > 0.0 {
                        let attract_strength =
                            boid_params.attract_target_factor / distance_to_closest_point;
                        let attract_vec =
                            (a_position - closest_point).normalize() * attract_strength;
                        // negate the vec to make the boid go toward the target
                        *final_velocity -= attract_vec;
                    // If the boid is inside the target
                    } else {
                        hp -= 1.0;
                    }
                }
            }
        }
        // If the object isn't a tile
        None => {
            // steer away from other boids
            let distance = a_position.distance(b_position);

            // if distance between boids is less than the threshold, steer away
            if distance > 0.0 {
                let avoid_strength = boid_params.avoid_factor / distance; // Using square of the distance to calculate strength
                *final_velocity += (a_position - b_position).normalize() * avoid_strength;
            }
        }
    }
}

fn spawn_boids(mut commands: Commands, map_aabb: Res<MapAabb>) {
    let mut rng = thread_rng();

    for _ in 0..1000 {
        commands.spawn((
            Name::new("Boid"),
            MovingObject {
                position: Position::new(Vec2::new(
                    rng.gen_range(-map_aabb.size.halfsize.x..map_aabb.size.halfsize.x),
                    rng.gen_range(-map_aabb.size.halfsize.y..map_aabb.size.halfsize.y) / 2.0,
                )),
                velocity: Velocity::new(Vec2::new(
                    rng.gen_range(-400.0..400.0),
                    rng.gen_range(-400.0..400.0),
                )),
                ..default()
            },
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(7.0)),
                    ..default()
                },
                ..default()
            },
            Boid::default(),
        ));
    }
}
