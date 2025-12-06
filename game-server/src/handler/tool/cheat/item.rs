use bevy_ecs::entity::Entity;
use protocol::game::tool::cheat_result::Result;
use crate::world::zone::Zone;

pub fn handle(entity: Entity, zone: &mut Zone, args: &[String]) -> (Result, String) {


    (Result::Ok, String::new())
}
