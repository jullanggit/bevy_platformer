use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use rand::{random, thread_rng, Rng};

use crate::{
    asset_loader::SpritesLoadingStates,
    map::{setup_map, MapAabb},
    physics::{MovingObject, MovingSpriteBundle, Position, Velocity, AABB},
    player::Player,
    quadtree::build_point_quadtree,
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

#[derive(Component)]
struct Boid;

#[derive(Resource, Reflect, Default)]
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

    pub avoid_player: bool,
    avoid_player_factor: f32,
    avoid_player_distance: f32,

    edge_avoidance_distance: f32,
    edge_avoidance_strength: f32,
}

fn move_boids(
    mut query: Query<(&mut MovingObject, Entity), (With<Boid>, Without<Player>)>,
    map_aabb: Res<MapAabb>,
    boid_params: Res<BoidParameters>,
    window: Query<&Window, With<PrimaryWindow>>,
    player_moving_object: Query<&MovingObject, With<Player>>,
) {
    // new
    let quadtree = build_point_quadtree(&query, &map_aabb);

    let mut boids = Vec::new();

    // collect all boids and the boids in their view range
    for (moving_object, entity) in &query {
        let position = moving_object.position;

        let mut other_boids = Vec::new();
        quadtree.query(&boid_params.view_distance_aabb, position, &mut other_boids);

        boids.push((entity, other_boids));
    }

    let mut rng = thread_rng();

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
                let [(a_moving_object, _), (b_moving_object, _)] =
                    query.get_many([a_entity, *b_entity]).unwrap();
                // define values for easier access
                let a_position = a_moving_object.position.value;
                let b_position = b_moving_object.position.value;
                let b_velocity = b_moving_object.velocity.value;

                // steer away from other boids
                let distance = a_position.distance(b_position);
                // if distance between boids is less than the threshold, steer away
                if distance > 0.0 {
                    let avoid_strength = boid_params.avoid_factor / distance; // Using square of the distance to calculate strength
                    final_velocity += (a_position - b_position).normalize() * avoid_strength;
                }

                // add to the accumulator
                (pos_acc + b_position, vel_acc + b_velocity, amount_acc + 1.0)
            },
        );
        // Get components of a_entity again, might be able to optimize
        let (mut a_moving_object, _) = query.get_mut(a_entity).unwrap();
        let a_position = a_moving_object.position.value;
        let a_velocity = a_moving_object.velocity.value;

        // Steer away from edges of the window
        let window = window.get_single().expect("No Primary window");
        let window_halfsize = 0.5 * Vec2::new(window.width(), window.height());

        if a_position.x < -window_halfsize.x + boid_params.edge_avoidance_distance {
            final_velocity.x += boid_params.edge_avoidance_strength
        } else if a_position.x > window_halfsize.x - boid_params.edge_avoidance_distance {
            final_velocity.x -= boid_params.edge_avoidance_strength
        }
        if a_position.y < -window_halfsize.y + boid_params.edge_avoidance_distance {
            final_velocity.y += boid_params.edge_avoidance_strength
        } else if a_position.y > window_halfsize.y - boid_params.edge_avoidance_distance {
            final_velocity.y -= boid_params.edge_avoidance_strength
        }

        // Avoid player
        if boid_params.avoid_player {
            let player_position = player_moving_object.single().position.value;
            let distance = a_position.distance(player_position);

            if distance < boid_params.avoid_player_distance && distance > 0.0 {
                let avoid_strength = boid_params.avoid_player_factor / distance; // Using square of the distance to calculate strength
                final_velocity += (a_position - player_position).normalize() * avoid_strength;
            }
        }

        // steer towards percieved center
        if boids_amount > 0.0 {
            let percieved_center = (total_position - a_position) / boids_amount;

            match boid_params.disperse {
                true => {
                    final_velocity += (a_position - percieved_center)
                        * boid_params.centering_factor
                        * boid_params.disperse_factor
                }
                false => {
                    final_velocity += (percieved_center - a_position) * boid_params.centering_factor
                }
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
            (rng.gen::<f32>() - 0.5) * boid_params.max_velocity * boid_params.random_factor;
        final_velocity.y +=
            (rng.gen::<f32>() - 0.5) * boid_params.max_velocity * boid_params.random_factor;

        a_moving_object.velocity.value += final_velocity * boid_params.multiplier;
    }
}

fn spawn_boids(mut commands: Commands, map_aabb: Res<MapAabb>) {
    let mut rng = thread_rng();

    let view_distance = 25.0;
    commands.insert_resource(BoidParameters {
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

        avoid_player: false,
        avoid_player_factor: 2000.0,
        avoid_player_distance: 32.0,

        edge_avoidance_distance: 10.0,
        edge_avoidance_strength: 10.0,
    });

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
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                ..default()
            },
            Boid,
        ));
    }
}
