use bevy_ecs::prelude::*;
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
