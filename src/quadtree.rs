use crate::{
    map::MapAabb,
    physics::{collides, intersects, MovingObject, Position, AABB},
};
use bevy::prelude::*;

#[derive(Debug)]
pub struct AabbQuadtree {
    boundary: AABB,
    center: Position,
    capacity: usize,
    objects: Vec<(Entity, AABB, Position)>,
    divided: bool,
    // Children
    nw: Option<Box<Self>>,
    ne: Option<Box<Self>>,
    sw: Option<Box<Self>>,
    se: Option<Box<Self>>,
}
impl AabbQuadtree {
    pub const fn new(boundary: AABB, center: Vec2, capacity: usize) -> Self {
        Self {
            boundary,
            center: Position::new(center),
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
        let center = self.center.value;
        let halfsize = Vec2::new(half_boundary.x, half_boundary.y);

        // Northwest
        let nw = AABB { halfsize };
        let nw_center = Vec2::new(center.x - half_boundary.x, center.y + half_boundary.y);
        self.nw = Some(Box::new(Self::new(nw, nw_center, self.capacity)));

        // Northeast
        let ne = AABB { halfsize };
        let ne_center = Vec2::new(center.x + half_boundary.x, center.y + half_boundary.y);
        self.ne = Some(Box::new(Self::new(ne, ne_center, self.capacity)));

        // Southwest
        let sw = AABB { halfsize };
        let sw_center = Vec2::new(center.x - half_boundary.x, center.y - half_boundary.y);
        self.sw = Some(Box::new(Self::new(sw, sw_center, self.capacity)));

        // Southeast
        let se = AABB { halfsize };
        let se_center = Vec2::new(center.x + half_boundary.x, center.y - half_boundary.y);
        self.se = Some(Box::new(Self::new(se, se_center, self.capacity)));

        self.divided = true;

        // implement redistrubuting objects
        let objects = std::mem::take(&mut self.objects);

        for (entity, aabb, position) in objects {
            self.insert(entity, aabb, position);
        }
    }

    pub fn insert(&mut self, entity: Entity, aabb: AABB, position: Position) -> bool {
        // Check if the aabb intersects the nodes boundary
        if !collides(&self.boundary, self.center, &aabb, position) {
            return false;
        }
        // If the node hasnt been subdivided yet
        if !self.divided {
            // and it still has capacity
            if self.objects.len() < self.capacity {
                // add it to the objects
                self.objects.push((entity, aabb, position));
                return true;
            }
            // if it doesnt have capacity anymore, subdivide
            self.subdivide();
        }

        // insert it into any child it intersects with// Define an array of mutable references to each quadrant
        let quadrants = [&mut self.nw, &mut self.ne, &mut self.sw, &mut self.se];

        let mut inserted = false;

        for quadrant in quadrants {
            if let Some(quadrant_ref) = quadrant.as_mut() {
                let quadrant_center = quadrant_ref.center;
                inserted |= quadrant_ref.insert(entity, aabb.clone(), position);
            }
        }

        inserted
    }

    pub fn query(&self, range: &AABB, position: Position, found: &mut Vec<Entity>) {
        // dont do anything if the range doesnt intersect with the nodes boundary
        if !collides(&self.boundary, self.center, range, position) {
            return;
        }

        // query child nodes
        if self.divided {
            self.nw.as_ref().unwrap().query(range, position, found);
            self.ne.as_ref().unwrap().query(range, position, found);
            self.sw.as_ref().unwrap().query(range, position, found);
            self.se.as_ref().unwrap().query(range, position, found);
        } else {
            // if it hasnt been divided, push all objects to found
            for (entity, _, _) in &self.objects {
                found.push(*entity);
            }
        }
    }
}

#[derive(Debug)]
pub struct PointQuadtree {
    boundary: AABB,
    center: Position,
    capacity: usize,
    objects: Vec<(Entity, Position)>,
    divided: bool,
    // Children
    nw: Option<Box<Self>>,
    ne: Option<Box<Self>>,
    sw: Option<Box<Self>>,
    se: Option<Box<Self>>,
}
impl PointQuadtree {
    pub const fn new(boundary: AABB, center: Vec2, capacity: usize) -> Self {
        Self {
            boundary,
            center: Position::new(center),
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
        let center = self.center.value;
        let halfsize = Vec2::new(half_boundary.x, half_boundary.y);

        // Northwest
        let nw = AABB { halfsize };
        let nw_center = Vec2::new(center.x - half_boundary.x, center.y + half_boundary.y);
        self.nw = Some(Box::new(Self::new(nw, nw_center, self.capacity)));

        // Northeast
        let ne = AABB { halfsize };
        let ne_center = Vec2::new(center.x + half_boundary.x, center.y + half_boundary.y);
        self.ne = Some(Box::new(Self::new(ne, ne_center, self.capacity)));

        // Southwest
        let sw = AABB { halfsize };
        let sw_center = Vec2::new(center.x - half_boundary.x, center.y - half_boundary.y);
        self.sw = Some(Box::new(Self::new(sw, sw_center, self.capacity)));

        // Southeast
        let se = AABB { halfsize };
        let se_center = Vec2::new(center.x + half_boundary.x, center.y - half_boundary.y);
        self.se = Some(Box::new(Self::new(se, se_center, self.capacity)));

        self.divided = true;

        // implement redistrubuting objects
        let objects = std::mem::take(&mut self.objects);

        for (entity, position) in objects {
            self.insert(entity, position);
        }
    }

    pub fn insert(&mut self, entity: Entity, position: Position) -> bool {
        // Check if the aabb intersects the nodes boundary
        if !intersects(&self.boundary, self.center, position) {
            return false;
        }
        // If the node hasnt been subdivided yet
        if !self.divided {
            // and it still has capacity
            if self.objects.len() < self.capacity {
                // add it to the objects
                self.objects.push((entity, position));
                return true;
            }
            // if it doesnt have capacity anymore, subdivide
            self.subdivide();
        }

        // Define an array of mutable references to each quadrant
        let quadrants = [&mut self.nw, &mut self.ne, &mut self.sw, &mut self.se];

        // insert it into any child it intersects with
        for quadrant in quadrants {
            if let Some(quadrant_ref) = quadrant.as_mut() {
                if quadrant_ref.insert(entity, position) {
                    return true;
                }
            }
        }

        false
    }

    pub fn query(&self, range: &AABB, position: Position, found: &mut Vec<Entity>) {
        // dont do anything if the range doesnt intersect with the nodes boundary
        if !collides(&self.boundary, self.center, range, position) {
            return;
        }

        // query child nodes
        if self.divided {
            self.nw.as_ref().unwrap().query(range, position, found);
            self.ne.as_ref().unwrap().query(range, position, found);
            self.sw.as_ref().unwrap().query(range, position, found);
            self.se.as_ref().unwrap().query(range, position, found);
        } else {
            // if it hasnt been divided, push all objects to found
            for (entity, _) in &self.objects {
                found.push(*entity);
            }
        }
    }
}

pub fn build_aabb_quadtree<'a, T>(items: T, map_aabb: &MapAabb) -> AabbQuadtree
where
    T: IntoIterator<Item = (&'a AABB, &'a MovingObject, Entity)>,
{
    let mut quadtree = AabbQuadtree::new(map_aabb.size.clone(), Vec2::ZERO, 2);
    items.into_iter().for_each(|item| {
        quadtree.insert(item.2, item.0.clone(), item.1.position);
    });
    quadtree
}
pub fn build_point_quadtree<'a, T>(items: T, map_aabb: &MapAabb) -> PointQuadtree
where
    T: IntoIterator<Item = (&'a MovingObject, Entity)>,
{
    let mut quadtree = PointQuadtree::new(map_aabb.size.clone(), Vec2::ZERO, 3);
    items.into_iter().for_each(|item| {
        quadtree.insert(item.1, item.0.position);
    });
    quadtree
}
