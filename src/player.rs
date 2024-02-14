use crate::map::TILE_SIZE;
use crate::physics::{
    Gravity, MovingObjectState, MovingSpriteBundle, Velocity, AABB, GRAVITY_CONSTANT,
};
use bevy::prelude::*;

const PLAYER_SPEED: f32 = 200.0;
pub const PLAYER_JUMP_FORCE: f32 = 40.0;
const JUMP_TIME: u8 = 15;
const PLAYER_TERMINAL_VELOCITY: f32 = 1000.0;

pub struct Playerplugin;
impl Plugin for Playerplugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            .register_type::<Jump>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (movement_controls));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
enum PlayerState {
    Standing,
    Walking,
    LoadingJump(Jump),
    #[default]
    Jumping,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Jump {
    pub jump_state: Option<u8>,
    jump_force: f32,
}
impl Jump {
    const fn new(jump_state: Option<u8>, jump_force: f32) -> Self {
        Self {
            jump_state,
            jump_force,
        }
    }
}

// Systems -- Startup
fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Name::new("Player"),
        MovingSpriteBundle {
            sprite_bundle: SpriteBundle {
                texture: asset_server.load("hkSprite.png"),
                sprite: Sprite::default(),
                ..default()
            },
            gravity: Gravity::new(GRAVITY_CONSTANT),
            aabb: AABB::new(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0)),
            ..default()
        },
        PlayerState::Standing,
    ));
}

// System -- Update
fn movement_controls(
    mut query: Query<
        (
            &mut Velocity,
            &mut PlayerState,
            &mut MovingObjectState,
            &mut Sprite,
        ),
        With<Player>,
    >,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let (mut velocity, mut player_state, mut moving_object_state, mut sprite) = query.single_mut();

    match player_state.as_mut() {
        PlayerState::Standing | PlayerState::Walking => {
            // left
            move_horizontal(
                1.0,
                &keyboard_input,
                &mut player_state,
                &mut sprite,
                &mut velocity,
                *moving_object_state,
                true,
            );

            // if jump key is pressed
            if keyboard_input.pressed(KeyCode::S) {
                initiate_jump(&mut player_state);
            }
        }
        // PlayerState::Walking => {
        // move_horizontal(
        // 1.0,
        // &keyboard_input,
        // &mut player_state,
        // &mut sprite,
        // &mut velocity,
        // &mut moving_object_state,
        // );
        //
        // if keyboard_input.pressed(KeyCode::S) {
        // initiate_jump(&mut player_state);
        // }
        // }
        PlayerState::LoadingJump(_jump) => {
            move_horizontal(
                0.5,
                &keyboard_input,
                &mut player_state,
                &mut sprite,
                &mut velocity,
                *moving_object_state,
                false,
            );

            if keyboard_input.pressed(KeyCode::S) {
                load_jump(&mut player_state);
            } else {
                execute_jump(&mut moving_object_state, &mut player_state, &mut velocity);
            }
        }
        PlayerState::Jumping => move_horizontal(
            0.7,
            &keyboard_input,
            &mut player_state,
            &mut sprite,
            &mut velocity,
            *moving_object_state,
            true,
        ),
    }
}

fn initiate_jump(player_state: &mut PlayerState) {
    *player_state = PlayerState::LoadingJump(Jump::new(Some(1), PLAYER_JUMP_FORCE));
}

fn load_jump(player_state: &mut PlayerState) {
    // loading jump
    if let PlayerState::LoadingJump(jump) = player_state {
        if let Some(load_time) = jump.jump_state.as_mut() {
            if *load_time < JUMP_TIME {
                // faster buildup at the start
                if *load_time == 1 {
                    *load_time += 1;
                }
                *load_time += 1;
            }
        }
    }
}

fn execute_jump(
    moving_object_state: &mut MovingObjectState,
    player_state: &mut PlayerState,
    velocity: &mut Velocity,
) {
    if let PlayerState::LoadingJump(jump) = player_state {
        if let Some(load_time) = jump.jump_state {
            velocity.value.y = PLAYER_JUMP_FORCE * load_time as f32;
        }
    }
    *player_state = PlayerState::Jumping;
    moving_object_state.on_ground = false;
}

fn move_horizontal(
    maneuverability: f32,
    keyboard_input: &Res<Input<KeyCode>>,
    player_state: &mut PlayerState,
    sprite: &mut Sprite,
    velocity: &mut Velocity,
    moving_object_state: MovingObjectState,
    change_state: bool,
) {
    // set state to standing if both or neither of the keys are pressed
    if keyboard_input.pressed(KeyCode::D) == keyboard_input.pressed(KeyCode::A) {
        if change_state {
            *player_state = PlayerState::Standing;
        }
        velocity.value.x = 0.0;
    }
    // left
    else if keyboard_input.pressed(KeyCode::A) {
        if change_state {
            *player_state = PlayerState::Walking;
        }
        if moving_object_state.pushes_left_wall {
            velocity.value.x = 0.0;
        } else {
            velocity.value.x = -PLAYER_SPEED * maneuverability;
            sprite.flip_x = true;
        }
        // right
    } else if keyboard_input.pressed(KeyCode::D) {
        if change_state {
            *player_state = PlayerState::Walking;
        }
        if moving_object_state.pushes_right_wall {
            velocity.value.x = 0.0;
        } else {
            velocity.value.x = PLAYER_SPEED * maneuverability;
            sprite.flip_x = false;
        }
    }

    // check if grounded
    if !moving_object_state.on_ground && change_state {
        *player_state = PlayerState::Jumping;
    }
}
