use bevy_ecs::prelude::*;
use nalgebra::{Point3, UnitVector2, Vector2};
use game_protocol::game::PTransform;

#[derive(Component, Clone, Copy)]
pub struct Transform {
    pub position: Point3<f32>,
    pub direction: UnitVector2<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Point3::default(),
            direction: UnitVector2::new_normalize(Vector2::new(1.0, 0.0))
        }
    }
}

impl Into<PTransform> for Transform {
    fn into(self) -> PTransform {
        PTransform {
            position: Some(self.position.into()),
            direction: Some(self.direction.into()),
        }
    }
}
