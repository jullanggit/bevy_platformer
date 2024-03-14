use bevy::{prelude::*, utils::HashMap};

use crate::physics::{MovingObject, MovingSpriteBundle, Position, AABB};

pub struct BoidPlugin;
impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boids)
            .add_systems(Update, move_boids);
    }
}

#[derive(Component)]
struct Boid;

fn move_boids(mut query: Query<(&mut MovingObject, Entity), With<Boid>>) {
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

    let distance_threshold = 100.0;
    let max_velocity = 100.0;

    let mut velocities = HashMap::with_capacity(query.iter().len());
    for (moving_object, entity) in &query {
        let mut final_velocity: Vec2;

        // steer towards percieved center (0.5%)
        let position = moving_object.position.value;
        let percieved_center = (total_position - position) / relative_boids_amount;
        final_velocity = (percieved_center - position) / 20.0;

        // steer away from other boids
        for (moving_object2, entity2) in &query {
            if entity == entity2 {
                continue;
            }
            let position2 = moving_object2.position.value;

            let distance = position.distance(position2);

            // if distance between boids is less than the threshold
            if distance < distance_threshold {
                final_velocity -= (position2 - position);
            }
        }

        // steer in the same direction as the other boids (10%)
        let percieved_velocity = (total_velocity - position) / relative_boids_amount;
        final_velocity += (percieved_velocity - position) / 10.0;

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
                    position: Position::new(Vec2::new(i as f32, j as f32)),
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
