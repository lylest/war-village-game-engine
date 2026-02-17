use crate::fighter::AttackData;
use crate::types::{Facing, Vec3, AABB};
use crate::weapon::WeaponData;

const BLOCK_DAMAGE_REDUCTION: f32 = 0.2; // blocked attacks deal 20% of normal damage
const HITSTUN_BASE_FRAMES: u32 = 12;
const LAUNCH_VELOCITY_Y: f32 = 8.0;

/// Result of a hit check between an attacker and defender.
#[derive(Debug, Clone)]
pub struct HitResult {
    pub damage: f32,
    pub knockback: Vec3,
    pub hitstun_frames: u32,
    pub was_blocked: bool,
    pub launches: bool,
}

/// Build the world-space hitbox for an attack given the attacker's position and facing.
pub fn attack_hitbox(
    attacker_pos: Vec3,
    facing: Facing,
    attack: &AttackData,
    _weapon: &WeaponData,
) -> AABB {
    let mut offset = attack.hitbox_offset;
    offset.x *= facing.sign();

    AABB::from_center(
        attacker_pos + offset,
        attack.hitbox_half_extents,
    )
}

/// Build the world-space hurtbox for a defender.
pub fn defender_hurtbox(defender_pos: Vec3, hurtbox_local: &AABB) -> AABB {
    hurtbox_local.translated(defender_pos)
}

/// Calculate damage and effects of a hit.
///
/// `combo_hits` is how many consecutive hits the defender has already taken
/// (used for combo scaling). `stale_multiplier` scales damage for repeated
/// use of the same attack (1.0 = fresh, lower = stale).
pub fn calculate_hit(
    attack: &AttackData,
    weapon: &WeaponData,
    attacker_defense: f32,
    defender_defense: f32,
    is_blocking: bool,
    attacker_pos: Vec3,
    defender_pos: Vec3,
    combo_hits: u32,
    stale_multiplier: f32,
) -> HitResult {
    // Combo scaling: each hit in a combo does 15% less damage, minimum 40%
    let combo_damage_scale = (1.0 - combo_hits as f32 * 0.15).max(0.4);
    // Combo hitstun scaling: each hit gives 25% less hitstun, minimum 30%
    let combo_hitstun_scale = (1.0 - combo_hits as f32 * 0.25).max(0.3);

    let raw_damage = weapon.base_damage * attack.damage_multiplier * stale_multiplier;
    let damage = if is_blocking {
        raw_damage * BLOCK_DAMAGE_REDUCTION * defender_defense
    } else {
        raw_damage * defender_defense * combo_damage_scale
    };

    // Knockback direction: push defender away from attacker
    let dir_x = if defender_pos.x >= attacker_pos.x {
        1.0
    } else {
        -1.0
    };

    // Knockback increases with combo hits to push fighters apart
    let combo_knockback_boost = 1.0 + combo_hits as f32 * 0.2;
    let knockback_magnitude = if is_blocking {
        attack.knockback_force * 0.3
    } else {
        attack.knockback_force * combo_knockback_boost
    };

    let launches = attack.launches && !is_blocking;

    let knockback = Vec3::new(
        dir_x * knockback_magnitude,
        if launches { LAUNCH_VELOCITY_Y } else { 0.0 },
        0.0,
    );

    let base_hitstun = if is_blocking {
        HITSTUN_BASE_FRAMES / 2
    } else {
        HITSTUN_BASE_FRAMES + (attack.knockback_force as u32)
    };
    let hitstun_frames = ((base_hitstun as f32) * combo_hitstun_scale) as u32;

    // Use attacker_defense to suppress the warning (it could scale damage in the future)
    let _ = attacker_defense;

    HitResult {
        damage,
        knockback,
        hitstun_frames,
        was_blocked: is_blocking,
        launches,
    }
}

/// Check if an attack hits a defender, and if so return the hit result.
pub fn check_hit(
    attacker_pos: Vec3,
    attacker_facing: Facing,
    attack: &AttackData,
    weapon: &WeaponData,
    attacker_defense: f32,
    defender_pos: Vec3,
    defender_hurtbox_local: &AABB,
    defender_defense: f32,
    defender_blocking: bool,
    combo_hits: u32,
    stale_multiplier: f32,
) -> Option<HitResult> {
    let hitbox = attack_hitbox(attacker_pos, attacker_facing, attack, weapon);
    let hurtbox = defender_hurtbox(defender_pos, defender_hurtbox_local);

    if hitbox.overlaps(&hurtbox) {
        Some(calculate_hit(
            attack,
            weapon,
            attacker_defense,
            defender_defense,
            defender_blocking,
            attacker_pos,
            defender_pos,
            combo_hits,
            stale_multiplier,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fighter::FighterData;
    use crate::fighter::FighterId;
    use crate::weapon::WeaponData;

    fn test_attack() -> &'static AttackData {
        &FighterData::get(FighterId::Kael).moveset.light_attack
    }

    fn test_weapon() -> &'static WeaponData {
        WeaponData::get(crate::weapon::WeaponType::Unarmed)
    }

    #[test]
    fn hit_connects_when_close() {
        let result = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            test_attack(),
            test_weapon(),
            1.0,
            Vec3::new(1.5, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            false,
            0,
            1.0,
        );
        assert!(result.is_some());
        let hit = result.unwrap();
        assert!(hit.damage > 0.0);
        assert!(!hit.was_blocked);
    }

    #[test]
    fn hit_misses_when_far() {
        let result = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            test_attack(),
            test_weapon(),
            1.0,
            Vec3::new(10.0, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            false,
            0,
            1.0,
        );
        assert!(result.is_none());
    }

    #[test]
    fn blocking_reduces_damage() {
        let unblocked = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            test_attack(),
            test_weapon(),
            1.0,
            Vec3::new(1.5, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            false,
            0,
            1.0,
        )
        .unwrap();

        let blocked = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            test_attack(),
            test_weapon(),
            1.0,
            Vec3::new(1.5, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            true,
            0,
            1.0,
        )
        .unwrap();

        assert!(blocked.damage < unblocked.damage);
        assert!(blocked.was_blocked);
        // Blocked damage should be ~20% of unblocked
        let ratio = blocked.damage / unblocked.damage;
        assert!((ratio - BLOCK_DAMAGE_REDUCTION).abs() < 0.01);
    }

    #[test]
    fn knockback_direction() {
        let hit = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            test_attack(),
            test_weapon(),
            1.0,
            Vec3::new(1.5, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            false,
            0,
            1.0,
        )
        .unwrap();

        // Defender is to the right, so knockback should push right (positive x)
        assert!(hit.knockback.x > 0.0);
    }

    #[test]
    fn launch_attack_sets_y_velocity() {
        let special = &FighterData::get(FighterId::Kael).moveset.aerial;
        assert!(special.launches);

        let hit = check_hit(
            Vec3::new(0.0, 0.0, 0.0),
            Facing::Right,
            special,
            test_weapon(),
            1.0,
            Vec3::new(1.0, 0.0, 0.0),
            &FighterData::get(FighterId::Kael).hurtbox,
            1.0,
            false,
            0,
            1.0,
        )
        .unwrap();

        assert!(hit.launches);
        assert!(hit.knockback.y > 0.0);
    }

    #[test]
    fn combo_scaling_reduces_damage() {
        let fresh_hit = calculate_hit(
            test_attack(), test_weapon(), 1.0, 1.0, false,
            Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 0.0, 0.0), 0, 1.0,
        );
        let combo_hit = calculate_hit(
            test_attack(), test_weapon(), 1.0, 1.0, false,
            Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 0.0, 0.0), 3, 1.0,
        );
        assert!(combo_hit.damage < fresh_hit.damage);
        assert!(combo_hit.hitstun_frames < fresh_hit.hitstun_frames);
    }

    #[test]
    fn stale_move_reduces_damage() {
        let fresh_hit = calculate_hit(
            test_attack(), test_weapon(), 1.0, 1.0, false,
            Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 0.0, 0.0), 0, 1.0,
        );
        let stale_hit = calculate_hit(
            test_attack(), test_weapon(), 1.0, 1.0, false,
            Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.5, 0.0, 0.0), 0, 0.6,
        );
        assert!(stale_hit.damage < fresh_hit.damage);
    }
}
