use bevy_ecs::prelude::*;
use std::time::Instant;

#[derive(Debug, PartialEq)]
pub enum StatusEffectKind {
    Buff,
    Debuff,
    Passive,
    Curse,
}

#[derive(macros::StatusEffect)]
pub enum StatusEffect {
    #[debuff] Stun,
    #[debuff] Slow { modifier: u8 },
    #[buff] Haste { modifier: u8 },
}

#[derive(Component)]
pub struct StatusEffectController {
    pub temporary_effects: Vec<(StatusEffect, Instant)>,
    pub permanent_effects: Vec<StatusEffect>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_effect() {
        let stun = StatusEffect::Stun;
        assert_eq!(stun.kind(), StatusEffectKind::Debuff);

        let slow = StatusEffect::Slow { modifier: 7 };
        assert_eq!(slow.kind(), StatusEffectKind::Debuff);

        let haste = StatusEffect::Haste { modifier: 4 };
        assert_eq!(haste.kind(), StatusEffectKind::Buff);
    }
}
