use bevy_ecs::prelude::*;
use nalgebra::{Point3, UnitVector2, Vector2};

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub position: Point3<f32>,
    pub direction: UnitVector2<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Point3::default(),
            direction: UnitVector2::new_normalize(Vector2::new(1.0, 0.0)),
        }
    }
}

impl Into<protocol::Transform> for &Transform {
    fn into(self) -> protocol::Transform {
        protocol::Transform {
            position: Some(self.position.into()),
            direction: Some(self.direction.into()),
        }
    }
}
