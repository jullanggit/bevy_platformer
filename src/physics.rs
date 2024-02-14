use bevy::prelude::*;

use crate::map::{
    collides_with_ceiling, collides_with_left_wall, collides_with_right_wall, has_ground, TileMap,
};

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .register_type::<Position>()
            .register_type::<AABB>()
            .register_type::<Gravity>()
            .register_type::<MovingObjectState>()
            .register_type::<MovingObject>()
            .add_systems(Update, (update_physics, apply_gravity, correct_collisions));
    }
}

pub const GRAVITY_CONSTANT: f32 = 9.8;

#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub value: Vec2,
}
impl Velocity {
    pub const fn new(value: Vec2) -> Self {
        Self { value }
    }
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Position {
    pub value: Vec2,
}
impl Position {
    pub const fn new(value: Vec2) -> Self {
        Self { value }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AABB {
    pub halfsize: Vec2,
}
impl AABB {
    pub const fn new(halfsize: Vec2) -> Self {
        Self { halfsize }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Gravity {
    pub force: f32,
}
impl Gravity {
    pub const fn new(force: f32) -> Self {
        Self { force }
    }
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct MovingObjectState {
    pub pushes_right_wall: bool,

    pub pushes_left_wall: bool,

    pub on_ground: bool,

    pub at_ceiling: bool,
}

#[derive(Component, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct MovingObject {
    pub old_position: Position,

    pub old_velocity: Velocity,

    pub old_state: MovingObjectState,

    pub aabb_offset: Vec2,
}

#[derive(Bundle, Default)]
pub struct MovingObjectBundle {
    transform: Transform,
    position: Position,
    velocity: Velocity,
    state: MovingObjectState,
    aabb: AABB,
    moving_object: MovingObject,
    gravity: Gravity,
}

#[derive(Bundle, Default)]
pub struct MovingSpriteBundle {
    pub position: Position,
    pub velocity: Velocity,
    pub state: MovingObjectState,
    pub aabb: AABB,
    pub moving_object: MovingObject,
    pub sprite_bundle: SpriteBundle,
    pub gravity: Gravity,
}

#[derive(Bundle, Default)]
pub struct MovingSpriteSheetBundle {
    pub position: Position,
    pub velocity: Velocity,
    pub state: MovingObjectState,
    pub aabb: AABB,
    pub moving_object: MovingObject,
    pub spritesheet_bundle: SpriteSheetBundle,
    pub gravity: Gravity,
}

fn update_physics(
    mut query: Query<(
        &mut MovingObject,
        &mut Transform,
        &Velocity,
        &MovingObjectState,
        &mut Position,
    )>,
    time: Res<Time>,
) {
    for (mut moving_object, mut transform, velocity, moving_object_state, mut position) in
        &mut query
    {
        moving_object.old_position = *position;
        moving_object.old_velocity = *velocity;
        moving_object.old_state = *moving_object_state;

        position.value += velocity.value * time.delta_seconds();

        transform.translation.x = position.value.x;
        transform.translation.y = position.value.y;
    }
}

fn correct_collisions(
    mut query: Query<(
        &mut Velocity,
        &AABB,
        &mut Position,
        &MovingObject,
        &mut MovingObjectState,
    )>,
    map: Res<TileMap>,
) {
    for (mut velocity, aabb, mut position, moving_object, mut moving_object_state) in &mut query {
        // if the character is moving upwards
        if velocity.value.y > 0.0 {
            if let Some(ceiling_y) = collides_with_ceiling(aabb, *position, &map) {
                position.value.y = ceiling_y - aabb.halfsize.y;
                velocity.value.y = 0.0;
                moving_object_state.at_ceiling = true;
            } else {
                moving_object_state.at_ceiling = false;
            }
        } else {
            moving_object_state.at_ceiling = false;
        }
        // if the character is moving downwards
        if velocity.value.y < 0.0 {
            // and has ground under him
            if let Some(ground_y) = has_ground(aabb, *position, &map) {
                position.value.y = ground_y + aabb.halfsize.y;
                velocity.value.y = 0.0;
                moving_object_state.on_ground = true;
            } else {
                moving_object_state.on_ground = false;
            }
        } else {
            moving_object_state.on_ground = false;
        }
        // if the character is moving left
        if velocity.value.x < 0.0 {
            // if there is a collision with a left wall
            if let Some(left_wall_x) = collides_with_left_wall(aabb, *position, &map) {
                velocity.value.x = 0.0;
                // if the character was overlapping with it the last frame
                if moving_object.old_position.value.x - aabb.halfsize.x >= left_wall_x {
                    position.value.x = left_wall_x + aabb.halfsize.x;
                    moving_object_state.pushes_left_wall = true;
                }
            } else {
                moving_object_state.pushes_left_wall = false;
            }
        } else {
            moving_object_state.pushes_left_wall = false;
        }
        // if the character is moving right
        if velocity.value.x > 0.0 {
            // if there is a collision with a right wall
            if let Some(right_wall_x) = collides_with_right_wall(aabb, *position, &map) {
                velocity.value.x = 0.0;
                // if the character was overlapping with it the last frame
                if moving_object.old_position.value.x + aabb.halfsize.x <= right_wall_x {
                    position.value.x = right_wall_x - aabb.halfsize.x;
                    moving_object_state.pushes_right_wall = true;
                }
            } else {
                moving_object_state.pushes_right_wall = false;
            }
        } else {
            moving_object_state.pushes_right_wall = false;
        }
    }
}

fn apply_gravity(mut query: Query<(&mut Velocity, &Gravity, &MovingObjectState)>) {
    for (mut velocity, gravity, state) in &mut query {
        if state.on_ground {
            velocity.value.y = 0.0;
        } else {
            velocity.value.y -= gravity.force;
        }
    }
}
