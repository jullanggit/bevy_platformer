use crate::asset_loader::load_assets;
use crate::map::TILE_SIZE;
use crate::physics::{Gravity, MovingObject, MovingSpriteBundle, AABB, GRAVITY_CONSTANT};
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
            .register_type::<Stretching>()
            .add_systems(Startup, spawn_player.after(load_assets))
            .add_systems(Update, movement_controls);
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

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Stretching {
    stretch_speed: f32,
    volume: f32,
    min_stretch: f32,
    pub currently_stretching: bool,
}

impl Stretching {
    pub const fn new(
        stretch_speed: f32,
        volume: f32,
        min_stretch: f32,
        currently_stretching: bool,
    ) -> Self {
        Self {
            stretch_speed,
            volume,
            min_stretch,
            currently_stretching,
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
                texture: asset_server.load("player.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                ..default()
            },
            gravity: Gravity::new(GRAVITY_CONSTANT),
            aabb: AABB::new(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0), Vec2::ZERO),
            moving_object: MovingObject {
                mass: 1.0,
                ..default()
            },
            ..default()
        },
        ImageScaleMode::Sliced(TextureSlicer {
            border: BorderRect::square(10.0),
            max_corner_scale: 1.0,
            ..default()
        }),
        PlayerState::Standing,
        Stretching::new(100.0, (TILE_SIZE / 2.0) * (TILE_SIZE / 2.0), 10.0, false),
    ));
}

// System -- Update
fn movement_controls(
    mut query: Query<
        (
            &mut MovingObject,
            &mut PlayerState,
            &mut Sprite,
            &mut AABB,
            &mut Stretching,
        ),
        With<Player>,
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut moving_object, mut player_state, mut sprite, mut aabb, mut stretching) =
        query.single_mut();

    match player_state.as_mut() {
        PlayerState::Standing | PlayerState::Walking => {
            // left
            move_horizontal(
                1.0,
                &keyboard_input,
                &mut player_state,
                &mut sprite,
                &mut moving_object,
                true,
            );

            // if jump key is pressed
            if keyboard_input.pressed(KeyCode::KeyS) {
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
                &mut moving_object,
                false,
            );

            if keyboard_input.pressed(KeyCode::KeyS) {
                load_jump(&mut player_state);
            } else {
                execute_jump(&mut moving_object, &mut player_state);
            }
        }
        PlayerState::Jumping => move_horizontal(
            0.7,
            &keyboard_input,
            &mut player_state,
            &mut sprite,
            &mut moving_object,
            true,
        ),
    }

    // Changing hitbox
    // horizontal
    if keyboard_input.pressed(KeyCode::KeyJ) {
        // prevent the player from getting to thin
        if aabb.halfsize.y > stretching.min_stretch {
            if !(moving_object.state.left && moving_object.state.right) {
                aabb.halfsize.x += stretching.stretch_speed * time.delta_seconds();
                aabb.halfsize.y = (stretching.volume / aabb.halfsize.x * 2.0) / 2.0;

                stretching.currently_stretching = true;
            }
        } else {
            aabb.halfsize.y = stretching.min_stretch;
        }
        // vertical
    } else if keyboard_input.pressed(KeyCode::KeyK) {
        // prevent the player from getting to thin
        if aabb.halfsize.x > stretching.min_stretch {
            if !(moving_object.state.ground && moving_object.state.ceiling) {
                aabb.halfsize.y += stretching.stretch_speed * time.delta_seconds();
                aabb.halfsize.x = (stretching.volume / aabb.halfsize.y * 2.0) / 2.0;

                stretching.currently_stretching = true;
            }
        } else {
            aabb.halfsize.x = stretching.min_stretch;
        }
    } else {
        stretching.currently_stretching = false;
    }
    sprite.custom_size = Some(aabb.halfsize * 2.0);
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

fn execute_jump(moving_object: &mut MovingObject, player_state: &mut PlayerState) {
    if let PlayerState::LoadingJump(jump) = player_state {
        if let Some(load_time) = jump.jump_state {
            moving_object.velocity.value.y = PLAYER_JUMP_FORCE * load_time as f32;
        }
    }
    *player_state = PlayerState::Jumping;
    moving_object.state.ground = false;
}

fn move_horizontal(
    maneuverability: f32,
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    player_state: &mut PlayerState,
    sprite: &mut Sprite,
    moving_object: &mut MovingObject,
    change_state: bool,
) {
    // set state to standing if both or neither of the keys are pressed
    if keyboard_input.pressed(KeyCode::KeyD) == keyboard_input.pressed(KeyCode::KeyA) {
        if change_state {
            *player_state = PlayerState::Standing;
        }
        moving_object.velocity.value.x = 0.0;
    }
    // left
    else if keyboard_input.pressed(KeyCode::KeyA) {
        if change_state {
            *player_state = PlayerState::Walking;
        }
        if moving_object.state.left {
            moving_object.velocity.value.x = 0.0;
        } else {
            moving_object.velocity.value.x = -PLAYER_SPEED * maneuverability;
            sprite.flip_x = true;
        }
        // right
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        if change_state {
            *player_state = PlayerState::Walking;
        }
        if moving_object.state.right {
            moving_object.velocity.value.x = 0.0;
        } else {
            moving_object.velocity.value.x = PLAYER_SPEED * maneuverability;
            sprite.flip_x = false;
        }
    }

    // check if grounded
    if !moving_object.state.ground && change_state {
        *player_state = PlayerState::Jumping;
    }
}
