use bevy::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .register_type::<Position>()
            .register_type::<AABB>()
            .register_type::<Gravity>()
            .register_type::<MovingObjectState>()
            .register_type::<MovingObject>()
            .add_systems(
                Update,
                (update_physics, apply_gravity, collisions, stop_movement),
            );
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

#[derive(Component, Clone, Copy, Default, Reflect, Debug)]
#[reflect(Component)]
pub struct Position {
    pub value: Vec2,
}
impl Position {
    pub const fn new(value: Vec2) -> Self {
        Self { value }
    }
}

#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
pub struct AABB {
    pub halfsize: Vec2,
    pub center: Vec2,
}
impl AABB {
    pub fn new(halfsize: Vec2, center: Vec2) -> Self {
        Self { halfsize, center }
    }

    pub fn contains(&self, other: &AABB) -> bool {
        // self.halfsize.x
        todo!()
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

#[derive(Component, Clone, Copy, Default, Reflect, Debug)]
#[reflect(Component)]
pub struct MovingObjectState {
    pub right: bool,
    pub left: bool,
    pub ground: bool,
    pub ceiling: bool,
}

#[derive(Component, Clone, Copy, Default, Reflect, Debug)]
#[reflect(Component)]
pub struct MovingObject {
    // timeless
    pub mass: f32,

    // current
    pub position: Position,
    pub velocity: Velocity,
    pub state: MovingObjectState,

    // old
    pub old_position: Position,
    pub old_velocity: Velocity,
    pub old_state: MovingObjectState,
}

#[derive(Bundle, Default)]
pub struct MovingObjectBundle {
    transform: Transform,
    aabb: AABB,
    moving_object: MovingObject,
    gravity: Gravity,
}

#[derive(Bundle, Default)]
pub struct MovingSpriteBundle {
    pub aabb: AABB,
    pub moving_object: MovingObject,
    pub sprite_bundle: SpriteBundle,
    pub gravity: Gravity,
}

#[derive(Bundle, Default)]
pub struct MovingSpriteSheetBundle {
    pub aabb: AABB,
    pub moving_object: MovingObject,
    pub spritesheet_bundle: SpriteSheetBundle,
    pub gravity: Gravity,
}
// Quadtree
pub struct Quadtree {
    boundary: AABB,
    capacity: usize,
    objects: Vec<Entity>,
    divided: bool,
    // Children
    nw: Option<Box<Quadtree>>,
    ne: Option<Box<Quadtree>>,
    sw: Option<Box<Quadtree>>,
    se: Option<Box<Quadtree>>,
}
impl Quadtree {
    pub fn new(boundary: AABB, capacity: usize) -> Self {
        Self {
            boundary,
            capacity,
            objects: Vec::new(),
            divided: false,
            nw: None,
            ne: None,
            sw: None,
            se: None,
        }
    }

    pub fn subdivide(&mut self) {
        let half_boundary = self.boundary.halfsize / 2.0;
        let center = self.boundary.center;

        let nw = AABB {
            center: Vec2::new(center.x - half_boundary.x, center.y + half_boundary.y),
            halfsize: Vec2::new(half_boundary.x, half_boundary.y),
        };
        self.nw = Some(Box::new(Quadtree::new(nw, self.capacity)));

        let ne = AABB {
            center: Vec2::new(center.x + half_boundary.x, center.y + half_boundary.y),
            halfsize: Vec2::new(half_boundary.x, half_boundary.y),
        };
        self.ne = Some(Box::new(Quadtree::new(ne, self.capacity)));

        let sw = AABB {
            center: Vec2::new(center.x - half_boundary.x, center.y - half_boundary.y),
            halfsize: Vec2::new(half_boundary.x, half_boundary.y),
        };
        self.sw = Some(Box::new(Quadtree::new(sw, self.capacity)));

        let se = AABB {
            center: Vec2::new(center.x + half_boundary.x, center.y - half_boundary.y),
            halfsize: Vec2::new(half_boundary.x, half_boundary.y),
        };
        self.se = Some(Box::new(Quadtree::new(se, self.capacity)));

        self.divided = true;
    }

    pub fn insert(&mut self, entity: Entity, aabb: &AABB) -> bool {
        // Check if the aabb fits inside the quadrtree's boundary
        // if !self.boundary
        todo!()
    }
}

fn update_physics(mut query: Query<(&mut MovingObject, &mut Transform)>, time: Res<Time>) {
    for (mut moving_object, mut transform) in &mut query {
        moving_object.old_position = moving_object.position;
        moving_object.old_velocity = moving_object.velocity;
        moving_object.old_state = moving_object.state;

        let velocity_value = moving_object.velocity.value;
        moving_object.position.value += velocity_value * time.delta_seconds();

        transform.translation.x = moving_object.position.value.x;
        transform.translation.y = moving_object.position.value.y;
    }
}

fn stop_movement(mut query: Query<&mut MovingObject>) {
    for mut moving_object in &mut query {
        if (moving_object.state.ceiling || moving_object.state.ground)
            && (!moving_object.old_state.ceiling || !moving_object.old_state.ground)
        {
            moving_object.velocity.value.y = 0.0;
        }
        if (moving_object.state.left || moving_object.state.right)
            && (!moving_object.old_state.left || !moving_object.old_state.right)
        {
            moving_object.velocity.value.x = 0.0;
        }
    }
}

fn collisions(mut query: Query<(&AABB, &mut MovingObject, Entity)>) {
    query.iter_mut().for_each(|(_, mut moving_object, _)| {
        // unset states
        moving_object.state.left = false;
        moving_object.state.right = false;
        moving_object.state.ground = false;
        moving_object.state.ceiling = false;
    });
    // generate over all combinations of 2
    let mut combinations = query.iter_combinations_mut::<2>();

    // iterate over combinations
    while let Some(
        [(a_aabb, mut a_moving_object, a_entity), (b_aabb, mut b_moving_object, b_entity)],
    ) = combinations.fetch_next()
    {
        // if both objects have a mass of 0 (are stationary), continue to next iteration
        if a_moving_object.mass == 0.0 && b_moving_object.mass == 0.0 {
            continue;
        }

        let a_pos = a_moving_object.position;
        let b_pos = b_moving_object.position;

        // if there is a collision
        if let Some(penetration_depth) = penetration_depth(a_aabb, a_pos, b_aabb, b_pos) {
            let total_mass = a_moving_object.mass + b_moving_object.mass;
            let a_ratio = a_moving_object.mass / total_mass;
            let b_ratio = b_moving_object.mass / total_mass;

            // determine which axis to adjust
            if penetration_depth.x.abs() < penetration_depth.y.abs() {
                // adjusting position
                a_moving_object.position.value.x += (penetration_depth.x * a_ratio);
                b_moving_object.position.value.x -= (penetration_depth.x * b_ratio);

                // setting horizontal states
                if penetration_depth.x >= 0.0 {
                    a_moving_object.state.left = true;
                    b_moving_object.state.right = true;
                } else {
                    a_moving_object.state.right = true;
                    b_moving_object.state.left = true;
                }
            } else {
                // adjusting position
                a_moving_object.position.value.y += (penetration_depth.y * a_ratio);
                b_moving_object.position.value.y -= (penetration_depth.y * b_ratio);

                if penetration_depth.y >= 0.0 {
                    a_moving_object.state.ground = true;
                    b_moving_object.state.ceiling = true;
                } else {
                    a_moving_object.state.ceiling = true;
                    b_moving_object.state.ground = true;
                }
            }
        }
    }
}

fn apply_gravity(mut query: Query<(&mut MovingObject, &Gravity)>) {
    for (mut moving_object, gravity) in &mut query {
        if moving_object.state.ground {
            moving_object.velocity.value.y = 0.0;
        } else {
            moving_object.velocity.value.y -= gravity.force;
        }
    }
}

fn penetration_depth(
    a_aabb: &AABB,
    a_pos: Position,
    b_aabb: &AABB,
    b_pos: Position,
) -> Option<Vec2> {
    if collides(a_aabb, a_pos, b_aabb, b_pos) {
        let x = if a_pos.value.x > b_pos.value.x {
            (b_pos.value.x + b_aabb.halfsize.x) - (a_pos.value.x - a_aabb.halfsize.x)
        } else {
            (b_pos.value.x - b_aabb.halfsize.x) - (a_pos.value.x + a_aabb.halfsize.x)
        };
        let y = if a_pos.value.y > b_pos.value.y {
            (b_pos.value.y + b_aabb.halfsize.y) - (a_pos.value.y - a_aabb.halfsize.y)
        } else {
            (b_pos.value.y - b_aabb.halfsize.y) - (a_pos.value.y + a_aabb.halfsize.y)
        };

        return Some(Vec2::new(x, y));
    }
    None
}

fn collides(a_aabb: &AABB, a_pos: Position, b_aabb: &AABB, b_pos: Position) -> bool {
    (a_pos.value.x - a_aabb.halfsize.x) < (b_pos.value.x + b_aabb.halfsize.x)
        && (a_pos.value.x + a_aabb.halfsize.x) > (b_pos.value.x - b_aabb.halfsize.x)
        && (a_pos.value.y + a_aabb.halfsize.y) > (b_pos.value.y - b_aabb.halfsize.y)
        && (a_pos.value.y - a_aabb.halfsize.y) < (b_pos.value.y + b_aabb.halfsize.y)
}
