use bevy_ecs::prelude::*;
use crate::physics::object::{KinematicBody, Transform};
use nalgebra::Vector2;

#[derive(Component)]
pub struct Visibility {
    visible: bool,
}

#[derive(Component)]
pub struct Vision {
    rays: Vec<Vector2<f32>>,
    sight: Vec<Entity>,
}

pub fn update_sight(
    mut eyes: Query<(&mut Vision, &Transform)>,
    objects: Query<(&Transform, &KinematicBody, &Visibility)>,
) {
    eyes.iter_mut().for_each(|eye| {

    });
}