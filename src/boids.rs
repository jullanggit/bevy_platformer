use bevy::{prelude::*, utils::HashMap};

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
    let distance_threshold = 100.0;
    let max_velocity = 400.0;
    let view_distance = 100.0;
    let view_distance_aabb = AABB::new(Vec2::splat(view_distance));

    let avoid_factor = 0.5;
    let centering_factor = 0.05;
    let matching_factor = 0.1;

    // new
    let quadtree = build_point_quadtree(&query, &map_aabb);

    let mut boids = Vec::new();

    // collect all boids, together with all other boids in its view range
    for (moving_object, entity) in &query {
        let position = moving_object.position;

        let mut other_boids = Vec::new();
        quadtree.query(&view_distance_aabb, position, &mut other_boids);

        boids.push((entity, other_boids));
    }

    for (a_entity, others) in boids {
        let final_velocity: Vec2;
        let (tota_position, total_velocity) = others.iter().fold(
            (Vec2::ZERO, Vec2::ZERO),
            |(pos_acc, vel_acc), (b_entity)| {
                // just return the accumulators if a and b are the same entity, essentialy skipping the
                // iteration
                if a_entity == *b_entity {
                    return (pos_acc, vel_acc);
                }
                // get components of both entities
                let [(mut a_moving_object, _), (mut b_moving_object, _)] =
                    query.get_many_mut([a_entity, *b_entity]).unwrap();
                // define values for easier access
                let a_position = a_moving_object.position.value;
                let a_velocity = a_moving_object.velocity.value;
                let b_position = b_moving_object.position.value;
                let b_velocity = b_moving_object.velocity.value;

                // add to the accumulator
                (pos_acc + b_position, vel_acc + b_velocity)
            },
        );
    }

    // old
    // for calculating "center of mass"
    let relative_boids_amount = query.iter().len() as f32 - 1.0;

    // calculate total position and velocity
    let (total_position, total_velocity) = query.iter().fold(
        (Vec2::ZERO, Vec2::ZERO),
        |(pos_acc, vel_acc), (moving_object, _)| {
            (
                pos_acc + moving_object.position.value,
                vel_acc + moving_object.velocity.value,
            )
        },
    );
    let mut velocities = HashMap::with_capacity(query.iter().len());
    for (moving_object, entity) in &query {
        let mut final_velocity: Vec2;

        // steer towards percieved center
        let position = moving_object.position.value;
        let percieved_center = (total_position - position) / relative_boids_amount;
        final_velocity = (percieved_center - position) * centering_factor;

        // steer away from other boids
        for (moving_object2, entity2) in &query {
            if entity == entity2 {
                continue;
            }
            let position2 = moving_object2.position.value;

            let distance = position.distance(position2);

            // if distance between boids is less than the threshold
            if distance < distance_threshold && distance > 0.0 {
                // Normalize the direction vector and scale it by the inverse of the distance (or another function to control the avoidance strength)
                final_velocity += (position - position2) * avoid_factor;
            }
        }

        // steer in the same direction as the other boids
        let velocity = moving_object.velocity.value;
        let percieved_velocity = (total_velocity - velocity) / relative_boids_amount;
        final_velocity += (percieved_velocity - velocity) * matching_factor;

        // Set maximal velocity
        if final_velocity.length() > max_velocity {
            final_velocity = final_velocity.normalize() * max_velocity;
        }
        velocities.insert(entity, final_velocity);
    }

    // set velocities
    for (mut moving_object, entity) in &mut query {
        moving_object.velocity.value = velocities.get(&entity).unwrap().clone();
    }
}

fn spawn_boids(mut commands: Commands) {
    for i in 0..10 {
        for j in 0..10 {
            commands.spawn((
                Name::new("Boid"),
                MovingObject {
                    position: Position::new(Vec2::new((i * 10) as f32, (j * 10) as f32)),
                    velocity: Velocity::new(Vec2::new(10.0, 0.0)),
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
}
