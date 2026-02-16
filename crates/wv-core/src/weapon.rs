use crate::types::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Katana,
    GreatSword,
    DualDaggers,
    Spear,
    WarHammer,
    Staff,
}

impl WeaponType {
    pub const ALL: [WeaponType; 6] = [
        WeaponType::Katana,
        WeaponType::GreatSword,
        WeaponType::DualDaggers,
        WeaponType::Spear,
        WeaponType::WarHammer,
        WeaponType::Staff,
    ];
}

impl std::fmt::Display for WeaponType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeaponType::Katana => write!(f, "Katana"),
            WeaponType::GreatSword => write!(f, "Great Sword"),
            WeaponType::DualDaggers => write!(f, "Dual Daggers"),
            WeaponType::Spear => write!(f, "Spear"),
            WeaponType::WarHammer => write!(f, "War Hammer"),
            WeaponType::Staff => write!(f, "Staff"),
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
    pub hitbox_half_extents: Vec3, // half-size of the attack hitbox
}

impl WeaponData {
    pub fn get(weapon_type: WeaponType) -> &'static WeaponData {
        match weapon_type {
            WeaponType::Katana => &KATANA,
            WeaponType::GreatSword => &GREAT_SWORD,
            WeaponType::DualDaggers => &DUAL_DAGGERS,
            WeaponType::Spear => &SPEAR,
            WeaponType::WarHammer => &WAR_HAMMER,
            WeaponType::Staff => &STAFF,
        }
    }
}

static KATANA: WeaponData = WeaponData {
    weapon_type: WeaponType::Katana,
    base_damage: 12.0,
    attack_speed: 1.2,
    range: 1.5,
    weight: 1.0,
    hitbox_half_extents: Vec3::new(0.75, 0.5, 0.3),
};

static GREAT_SWORD: WeaponData = WeaponData {
    weapon_type: WeaponType::GreatSword,
    base_damage: 20.0,
    attack_speed: 0.7,
    range: 2.0,
    weight: 2.5,
    hitbox_half_extents: Vec3::new(1.0, 0.8, 0.4),
};

static DUAL_DAGGERS: WeaponData = WeaponData {
    weapon_type: WeaponType::DualDaggers,
    base_damage: 8.0,
    attack_speed: 1.6,
    range: 1.0,
    weight: 0.5,
    hitbox_half_extents: Vec3::new(0.5, 0.4, 0.3),
};

static SPEAR: WeaponData = WeaponData {
    weapon_type: WeaponType::Spear,
    base_damage: 14.0,
    attack_speed: 1.0,
    range: 2.5,
    weight: 1.5,
    hitbox_half_extents: Vec3::new(1.25, 0.3, 0.3),
};

static WAR_HAMMER: WeaponData = WeaponData {
    weapon_type: WeaponType::WarHammer,
    base_damage: 25.0,
    attack_speed: 0.5,
    range: 1.3,
    weight: 3.0,
    hitbox_half_extents: Vec3::new(0.65, 0.7, 0.4),
};

static STAFF: WeaponData = WeaponData {
    weapon_type: WeaponType::Staff,
    base_damage: 10.0,
    attack_speed: 1.1,
    range: 2.2,
    weight: 1.2,
    hitbox_half_extents: Vec3::new(1.1, 0.4, 0.3),
};
