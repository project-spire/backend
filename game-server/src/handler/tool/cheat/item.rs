use crate::task::{dispatch, Task};
use bevy_ecs::prelude::*;
use protocol::game::tool::cheat_result::Result;

pub fn handle(world: &mut World, entity: Entity, args: &[String]) -> Option<(Result, String)> {
    let task = Task::serial(async {
        let mut conn = db::conn().await?;

        //TODO: Insert a item

        Ok(())
    }).on_complete(|error, entity, world| {
        if let Some(error) = error {
            //TODO: Send fail message
            return;
        }
    });
    
    dispatch(world, entity, task);

    None
}
