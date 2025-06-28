use bevy_ecs::prelude::*;
use nalgebra::{distance, Point2, Vector2};
use crate::physics::object::*;

pub enum CollisionShape {
    Rectangle { w: f32, h: f32 },
    Circle { radius: f32 },
}

#[derive(Component)]
pub struct Collision {
    pub a: Entity,
    pub b: Entity,
}

pub fn dotcast(dot: Point2<f32>, p: Point2<f32>, c: &CollisionShape) -> bool {
    match c {
        CollisionShape::Rectangle { w, h } => {
            p.x - w <= dot.x && dot.x <= p.x + w &&
            p.y - h <= dot.y && dot.y <= p.y + h
        }
        CollisionShape::Circle { radius } => {
            distance(&dot, &p) <= *radius
        }
    }
}

pub fn raycast(
    p: Point2<f32>,
    v: Vector2<f32>,
    static_bodies: Query<&StaticBody>,
    kinematic_bodies: Query<&KinematicBody>,
) -> Option<Entity> {
    None
}

pub fn raycast_many(
    p: Point2<f32>,
    v: Vector2<f32>,
    static_bodies: Query<&StaticBody>,
    kinematic_bodies: Query<&KinematicBody>,
) -> Vec<Entity> {
    let mut result: Vec<Entity> = Vec::new();
    
    static_bodies.iter().for_each(|body| {
        
    });
    
    kinematic_bodies.iter().for_each(|body| {
        
    });
    
    result
}
