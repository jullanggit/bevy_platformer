use bevy::{prelude::*, utils::HashMap};
use rand::{random, thread_rng, Rng};

use crate::{
    map::MapAabb,
    physics::{MovingObject, MovingSpriteBundle, Position, Velocity, AABB},
    quadtree::build_point_quadtree,
};

pub struct BoidPlugin;
impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boids)
            .add_systems(Update, move_boids);
    }
}

#[derive(Component)]
struct Boid;

fn move_boids(mut query: Query<(&mut MovingObject, Entity), With<Boid>>, map_aabb: Res<MapAabb>) {
    let distance_threshold = 20.0;
    let max_velocity = 400.0;
    let min_velocity = 20.0;
    let view_distance = 100.0;
    let view_distance_aabb = AABB::new(Vec2::splat(view_distance));

    let avoid_factor = 0.2;
    let centering_factor = 0.05;
    let matching_factor = 0.1;

    // new
    let quadtree = build_point_quadtree(&query, &map_aabb);

    let mut boids = Vec::new();

    // collect all boids and the boids in their view range
    for (moving_object, entity) in &query {
        let position = moving_object.position;

        let mut other_boids = Vec::new();
        quadtree.query(&view_distance_aabb, position, &mut other_boids);

        boids.push((entity, other_boids));
    }

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
                if distance < distance_threshold && distance > 0.0 {
                    final_velocity += (a_position - b_position) * avoid_factor;
                }

                // add to the accumulator
                (pos_acc + b_position, vel_acc + b_velocity, amount_acc + 1.0)
            },
        );
        // Get components of a_entity again, might be able to optimize
        let (mut a_moving_object, _) = query.get_mut(a_entity).unwrap();
        let a_position = a_moving_object.position.value;
        let a_velocity = a_moving_object.velocity.value;

        // steer towards percieved center
        if boids_amount > 0.0 {
            let percieved_center = (total_position - a_position) / boids_amount;
            final_velocity += (percieved_center - a_position) * centering_factor;
        }

        // steer in the same direction as the other boids
        if boids_amount > 0.0 {
            let percieved_velocity = (total_velocity - a_velocity) / boids_amount;
            final_velocity += (percieved_velocity - a_velocity) * matching_factor;
        }

        // Normalize velocity
        let final_velocity_length = final_velocity.length();
        if final_velocity_length > max_velocity {
            final_velocity = final_velocity.normalize() * max_velocity;
        } else if final_velocity_length < min_velocity {
            final_velocity = final_velocity.normalize() * min_velocity;
        }
        dbg!(final_velocity.length());
        a_moving_object.velocity.value = final_velocity;
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
                    rng.gen_range(-map_aabb.size.halfsize.y..map_aabb.size.halfsize.y),
                )),
                velocity: Velocity::new(Vec2::new(
                    rng.gen_range(0.0..300.0),
                    rng.gen_range(0.0..300.0),
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
