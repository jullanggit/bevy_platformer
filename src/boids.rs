use bevy::prelude::*;

use crate::physics::{MovingObject, AABB};

#[derive(Component)]
struct Boid;

fn move_boids(mut query: Query<(&AABB, &mut MovingObject), With<Boid>>) {
    // for calculating "center of mass"
    let boids_amount = query.iter().len();
    let total_position: Vec2 = query
        .iter()
        .map(|(_, moving_object)| moving_object.position.value)
        .sum();

    for (aabb, mut moving_object) in &mut query {
        // steer towards center
        let rule1 = (total_position - moving_object.position.value) / (boids_amount - 1) as f32;

        // steer away from other boids
        let distance_threshold = 
    }
}

fn rule2(aabb: &AABB, moving_object: &MovingObject) -> Vec2 {}
fn rule3(aabb: &AABB, moving_object: &MovingObject) -> Vec2 {}
