use crate::types::{AABB, Vec3};
use crate::weapon::WeaponType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FighterId {
    Kenzo,
    Mira,
    Thane,
    Yuki,
    Drago,
    Sage,
}

impl FighterId {
    pub const ALL: [FighterId; 6] = [
        FighterId::Kenzo,
        FighterId::Mira,
        FighterId::Thane,
        FighterId::Yuki,
        FighterId::Drago,
        FighterId::Sage,
    ];
}

impl std::fmt::Display for FighterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FighterId::Kenzo => write!(f, "Kenzo"),
            FighterId::Mira => write!(f, "Mira"),
            FighterId::Thane => write!(f, "Thane"),
            FighterId::Yuki => write!(f, "Yuki"),
            FighterId::Drago => write!(f, "Drago"),
            FighterId::Sage => write!(f, "Sage"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FighterStyle {
    Balanced,
    Aggressive,
    Defensive,
    Speed,
    Power,
    Technical,
}

#[derive(Debug, Clone)]
pub struct AttackData {
    pub name: &'static str,
    pub damage_multiplier: f32,   // applied on top of weapon base damage
    pub startup_frames: u32,
    pub active_frames: u32,
    pub recovery_frames: u32,
    pub knockback_force: f32,
    pub hitbox_offset: Vec3,      // relative to fighter position + facing
    pub hitbox_half_extents: Vec3,
    pub launches: bool,           // sends opponent airborne
}

impl AttackData {
    pub fn total_frames(&self) -> u32 {
        self.startup_frames + self.active_frames + self.recovery_frames
    }
}

#[derive(Debug, Clone)]
pub struct MoveSet {
    pub light_attack: AttackData,
    pub heavy_attack: AttackData,
    pub special_attack: AttackData,
    pub combo_finisher: AttackData, // final hit of a combo chain
}

#[derive(Debug, Clone)]
pub struct FighterData {
    pub id: FighterId,
    pub style: FighterStyle,
    pub max_health: f32,
    pub max_stamina: f32,
    pub move_speed: f32,
    pub dash_speed: f32,
    pub dash_frames: u32,
    pub defense: f32,          // damage reduction multiplier (lower = less damage taken)
    pub default_weapon: WeaponType,
    pub moveset: MoveSet,
    pub hurtbox: AABB,         // body hurtbox relative to position (origin at feet)
}

impl FighterData {
    pub fn get(id: FighterId) -> &'static FighterData {
        match id {
            FighterId::Kenzo => &KENZO,
            FighterId::Mira => &MIRA,
            FighterId::Thane => &THANE,
            FighterId::Yuki => &YUKI,
            FighterId::Drago => &DRAGO,
            FighterId::Sage => &SAGE,
        }
    }
}

// -- Fighter Definitions --

static KENZO: FighterData = FighterData {
    id: FighterId::Kenzo,
    style: FighterStyle::Balanced,
    max_health: 100.0,
    max_stamina: 100.0,
    move_speed: 5.0,
    dash_speed: 12.0,
    dash_frames: 10,
    defense: 1.0,
    default_weapon: WeaponType::Katana,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Slash",
            damage_multiplier: 1.0,
            startup_frames: 4,
            active_frames: 3,
            recovery_frames: 6,
            knockback_force: 3.0,
            hitbox_offset: Vec3::new(1.2, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.6, 0.4, 0.3),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Overhead Slash",
            damage_multiplier: 1.8,
            startup_frames: 10,
            active_frames: 4,
            recovery_frames: 12,
            knockback_force: 6.0,
            hitbox_offset: Vec3::new(1.0, 1.0, 0.0),
            hitbox_half_extents: Vec3::new(0.7, 0.6, 0.3),
            launches: false,
        },
        special_attack: AttackData {
            name: "Rising Dragon",
            damage_multiplier: 2.0,
            startup_frames: 6,
            active_frames: 5,
            recovery_frames: 18,
            knockback_force: 8.0,
            hitbox_offset: Vec3::new(0.8, 1.2, 0.0),
            hitbox_half_extents: Vec3::new(0.5, 0.8, 0.3),
            launches: true,
        },
        combo_finisher: AttackData {
            name: "Blade Storm",
            damage_multiplier: 2.5,
            startup_frames: 3,
            active_frames: 6,
            recovery_frames: 15,
            knockback_force: 10.0,
            hitbox_offset: Vec3::new(1.0, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.8, 0.6, 0.4),
            launches: false,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.4, 0.0, -0.3),
        max: Vec3::new(0.4, 1.8, 0.3),
    },
};

static MIRA: FighterData = FighterData {
    id: FighterId::Mira,
    style: FighterStyle::Speed,
    max_health: 85.0,
    max_stamina: 120.0,
    move_speed: 6.5,
    dash_speed: 15.0,
    dash_frames: 8,
    defense: 1.1,
    default_weapon: WeaponType::DualDaggers,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Twin Stab",
            damage_multiplier: 0.8,
            startup_frames: 3,
            active_frames: 2,
            recovery_frames: 4,
            knockback_force: 2.0,
            hitbox_offset: Vec3::new(0.8, 0.7, 0.0),
            hitbox_half_extents: Vec3::new(0.4, 0.3, 0.3),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Cross Slash",
            damage_multiplier: 1.5,
            startup_frames: 7,
            active_frames: 3,
            recovery_frames: 9,
            knockback_force: 5.0,
            hitbox_offset: Vec3::new(0.9, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.5, 0.5, 0.3),
            launches: false,
        },
        special_attack: AttackData {
            name: "Shadow Dance",
            damage_multiplier: 1.8,
            startup_frames: 5,
            active_frames: 6,
            recovery_frames: 14,
            knockback_force: 4.0,
            hitbox_offset: Vec3::new(1.0, 0.6, 0.0),
            hitbox_half_extents: Vec3::new(0.6, 0.5, 0.4),
            launches: false,
        },
        combo_finisher: AttackData {
            name: "Thousand Cuts",
            damage_multiplier: 2.2,
            startup_frames: 2,
            active_frames: 8,
            recovery_frames: 12,
            knockback_force: 7.0,
            hitbox_offset: Vec3::new(0.8, 0.7, 0.0),
            hitbox_half_extents: Vec3::new(0.6, 0.5, 0.4),
            launches: false,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.3, 0.0, -0.3),
        max: Vec3::new(0.3, 1.6, 0.3),
    },
};

static THANE: FighterData = FighterData {
    id: FighterId::Thane,
    style: FighterStyle::Defensive,
    max_health: 120.0,
    max_stamina: 80.0,
    move_speed: 3.5,
    dash_speed: 9.0,
    dash_frames: 12,
    defense: 0.8,
    default_weapon: WeaponType::GreatSword,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Broad Slash",
            damage_multiplier: 1.2,
            startup_frames: 7,
            active_frames: 5,
            recovery_frames: 10,
            knockback_force: 5.0,
            hitbox_offset: Vec3::new(1.4, 0.9, 0.0),
            hitbox_half_extents: Vec3::new(0.8, 0.6, 0.4),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Earthshatter",
            damage_multiplier: 2.2,
            startup_frames: 14,
            active_frames: 6,
            recovery_frames: 16,
            knockback_force: 10.0,
            hitbox_offset: Vec3::new(1.2, 0.5, 0.0),
            hitbox_half_extents: Vec3::new(1.0, 0.8, 0.5),
            launches: true,
        },
        special_attack: AttackData {
            name: "Shield Bash",
            damage_multiplier: 1.5,
            startup_frames: 8,
            active_frames: 4,
            recovery_frames: 12,
            knockback_force: 12.0,
            hitbox_offset: Vec3::new(0.8, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.5, 0.6, 0.4),
            launches: false,
        },
        combo_finisher: AttackData {
            name: "Titan Cleave",
            damage_multiplier: 3.0,
            startup_frames: 6,
            active_frames: 5,
            recovery_frames: 20,
            knockback_force: 14.0,
            hitbox_offset: Vec3::new(1.2, 1.0, 0.0),
            hitbox_half_extents: Vec3::new(0.9, 0.7, 0.5),
            launches: true,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.5, 0.0, -0.3),
        max: Vec3::new(0.5, 2.0, 0.3),
    },
};

static YUKI: FighterData = FighterData {
    id: FighterId::Yuki,
    style: FighterStyle::Technical,
    max_health: 90.0,
    max_stamina: 110.0,
    move_speed: 5.5,
    dash_speed: 13.0,
    dash_frames: 9,
    defense: 1.0,
    default_weapon: WeaponType::Spear,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Thrust",
            damage_multiplier: 1.0,
            startup_frames: 5,
            active_frames: 3,
            recovery_frames: 7,
            knockback_force: 3.5,
            hitbox_offset: Vec3::new(1.8, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.9, 0.3, 0.2),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Sweep",
            damage_multiplier: 1.6,
            startup_frames: 9,
            active_frames: 5,
            recovery_frames: 11,
            knockback_force: 6.0,
            hitbox_offset: Vec3::new(1.5, 0.3, 0.0),
            hitbox_half_extents: Vec3::new(1.0, 0.4, 0.5),
            launches: false,
        },
        special_attack: AttackData {
            name: "Dragon Spear",
            damage_multiplier: 2.2,
            startup_frames: 8,
            active_frames: 4,
            recovery_frames: 16,
            knockback_force: 9.0,
            hitbox_offset: Vec3::new(2.0, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(1.2, 0.4, 0.3),
            launches: true,
        },
        combo_finisher: AttackData {
            name: "Piercing Rain",
            damage_multiplier: 2.4,
            startup_frames: 4,
            active_frames: 7,
            recovery_frames: 14,
            knockback_force: 8.0,
            hitbox_offset: Vec3::new(1.6, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(1.0, 0.5, 0.4),
            launches: false,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.35, 0.0, -0.3),
        max: Vec3::new(0.35, 1.7, 0.3),
    },
};

static DRAGO: FighterData = FighterData {
    id: FighterId::Drago,
    style: FighterStyle::Power,
    max_health: 130.0,
    max_stamina: 70.0,
    move_speed: 3.0,
    dash_speed: 8.0,
    dash_frames: 14,
    defense: 0.75,
    default_weapon: WeaponType::WarHammer,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Hammer Swing",
            damage_multiplier: 1.3,
            startup_frames: 8,
            active_frames: 4,
            recovery_frames: 10,
            knockback_force: 6.0,
            hitbox_offset: Vec3::new(1.0, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.6, 0.5, 0.4),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Ground Pound",
            damage_multiplier: 2.5,
            startup_frames: 16,
            active_frames: 6,
            recovery_frames: 18,
            knockback_force: 14.0,
            hitbox_offset: Vec3::new(0.8, 0.3, 0.0),
            hitbox_half_extents: Vec3::new(1.0, 0.6, 0.6),
            launches: true,
        },
        special_attack: AttackData {
            name: "Berserker Charge",
            damage_multiplier: 2.0,
            startup_frames: 10,
            active_frames: 8,
            recovery_frames: 20,
            knockback_force: 12.0,
            hitbox_offset: Vec3::new(1.2, 0.7, 0.0),
            hitbox_half_extents: Vec3::new(0.8, 0.6, 0.4),
            launches: false,
        },
        combo_finisher: AttackData {
            name: "Meteor Smash",
            damage_multiplier: 3.5,
            startup_frames: 8,
            active_frames: 5,
            recovery_frames: 24,
            knockback_force: 18.0,
            hitbox_offset: Vec3::new(1.0, 1.0, 0.0),
            hitbox_half_extents: Vec3::new(0.9, 0.8, 0.5),
            launches: true,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.5, 0.0, -0.35),
        max: Vec3::new(0.5, 2.1, 0.35),
    },
};

static SAGE: FighterData = FighterData {
    id: FighterId::Sage,
    style: FighterStyle::Aggressive,
    max_health: 95.0,
    max_stamina: 100.0,
    move_speed: 5.0,
    dash_speed: 11.0,
    dash_frames: 10,
    defense: 0.95,
    default_weapon: WeaponType::Staff,
    moveset: MoveSet {
        light_attack: AttackData {
            name: "Staff Strike",
            damage_multiplier: 0.9,
            startup_frames: 4,
            active_frames: 3,
            recovery_frames: 5,
            knockback_force: 3.0,
            hitbox_offset: Vec3::new(1.4, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.7, 0.4, 0.3),
            launches: false,
        },
        heavy_attack: AttackData {
            name: "Arcane Blast",
            damage_multiplier: 1.8,
            startup_frames: 10,
            active_frames: 5,
            recovery_frames: 12,
            knockback_force: 7.0,
            hitbox_offset: Vec3::new(1.6, 0.8, 0.0),
            hitbox_half_extents: Vec3::new(0.8, 0.6, 0.4),
            launches: false,
        },
        special_attack: AttackData {
            name: "Mystic Vortex",
            damage_multiplier: 2.0,
            startup_frames: 7,
            active_frames: 8,
            recovery_frames: 15,
            knockback_force: 6.0,
            hitbox_offset: Vec3::new(1.2, 0.6, 0.0),
            hitbox_half_extents: Vec3::new(1.0, 0.8, 0.6),
            launches: false,
        },
        combo_finisher: AttackData {
            name: "Spirit Eruption",
            damage_multiplier: 2.8,
            startup_frames: 5,
            active_frames: 6,
            recovery_frames: 16,
            knockback_force: 11.0,
            hitbox_offset: Vec3::new(1.0, 1.0, 0.0),
            hitbox_half_extents: Vec3::new(0.9, 0.7, 0.5),
            launches: true,
        },
    },
    hurtbox: AABB {
        min: Vec3::new(-0.4, 0.0, -0.3),
        max: Vec3::new(0.4, 1.8, 0.3),
    },
};
