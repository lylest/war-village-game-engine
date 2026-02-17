use crate::types::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Unarmed,
    SwordAndShield,
    Magic,
}

impl WeaponType {
    pub const ALL: [WeaponType; 3] = [
        WeaponType::Unarmed,
        WeaponType::SwordAndShield,
        WeaponType::Magic,
    ];
}

impl std::fmt::Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponType::Unarmed => write!(f, "Unarmed"),
            WeaponType::SwordAndShield => write!(f, "Sword & Shield"),
            WeaponType::Magic => write!(f, "Magic"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeaponData {
    pub weapon_type: WeaponType,
    pub base_damage: f32,
    pub attack_speed: f32,   // multiplier (1.0 = normal)
    pub range: f32,          // hitbox reach in front of fighter
    pub weight: f32,         // affects knockback dealt
    pub hitbox_half_extents: Vec3,
}

impl WeaponData {
    pub fn get(weapon_type: WeaponType) -> &'static WeaponData {
        match weapon_type {
            WeaponType::Unarmed => &UNARMED,
            WeaponType::SwordAndShield => &SWORD_AND_SHIELD,
            WeaponType::Magic => &MAGIC,
        }
    }
}

// Unarmed: fast attacks, short range, low base damage
// Damage tuned so fights last 15-25 hits (MK-style pacing)
static UNARMED: WeaponData = WeaponData {
    weapon_type: WeaponType::Unarmed,
    base_damage: 3.0,
    attack_speed: 1.4,
    range: 1.0,
    weight: 0.6,
    hitbox_half_extents: Vec3::new(0.5, 0.4, 0.3),
};

// Sword & Shield: balanced damage, good range, heavy knockback
static SWORD_AND_SHIELD: WeaponData = WeaponData {
    weapon_type: WeaponType::SwordAndShield,
    base_damage: 4.5,
    attack_speed: 0.9,
    range: 1.8,
    weight: 2.0,
    hitbox_half_extents: Vec3::new(0.9, 0.6, 0.4),
};

// Magic: medium damage, long range, moderate speed
static MAGIC: WeaponData = WeaponData {
    weapon_type: WeaponType::Magic,
    base_damage: 3.5,
    attack_speed: 1.1,
    range: 2.2,
    weight: 1.0,
    hitbox_half_extents: Vec3::new(0.8, 0.5, 0.4),
};
