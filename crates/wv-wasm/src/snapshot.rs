use serde::Serialize;
use wv_core::fighter::{AnimationSet, MoveSet};
use wv_core::game::{GamePhase, GameState};
use wv_core::state_machine::{ActiveAttack, AttackPhase, FighterState};

#[derive(Serialize)]
pub struct Vec3Snapshot {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize)]
pub struct AttackSnapshot {
    pub attack_type: &'static str,
    pub phase: &'static str,
    pub phase_num: u8,
    pub hit_connected: bool,
    pub frame_counter: u32,
    pub total_frames: u32,
}

#[derive(Serialize)]
pub struct FighterSnapshot {
    pub fighter_id: String,
    pub weapon_type: String,
    pub position: Vec3Snapshot,
    pub velocity: Vec3Snapshot,
    pub health: f32,
    pub max_health: f32,
    pub health_pct: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub stamina_pct: f32,
    pub facing: &'static str,
    pub state: String,
    pub state_num: u8,
    pub attack: Option<AttackSnapshot>,
    pub round_wins: u32,
    pub grounded: bool,
    pub anim_dir: &'static str,
    pub current_anim: String,
}

#[derive(Serialize)]
pub struct ArenaBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

#[derive(Serialize)]
pub struct GameSnapshot {
    pub phase: &'static str,
    pub frame: u32,
    pub round_timer: f32,
    pub current_round: u32,
    pub countdown_display: &'static str,
    pub last_hit_info: Option<String>,
    pub winner: Option<u8>,
    pub fighters: [FighterSnapshot; 2],
}

/// Full animation data for a fighter (returned by fighter_animations()).
#[derive(Serialize)]
pub struct AnimationSetSnapshot {
    pub dir: &'static str,
    pub idle: &'static str,
    pub run: &'static str,
    pub run_backward: &'static str,
    pub strafe_left: &'static str,
    pub strafe_right: &'static str,
    pub block: &'static str,
    pub hit_reaction: &'static str,
    pub knockdown: &'static str,
    pub getting_up: &'static str,
    pub death: &'static str,
    pub sweep_fall: &'static str,
    pub light_attack: &'static str,
    pub heavy_attack: &'static str,
    pub special_attack: &'static str,
    pub mid_kick: &'static str,
    pub low_kick: &'static str,
    pub aerial: &'static str,
    pub combo_finisher: &'static str,
    pub super_attack: &'static str,
}

pub fn animation_set_snapshot(anims: &AnimationSet, moveset: &MoveSet) -> AnimationSetSnapshot {
    AnimationSetSnapshot {
        dir: anims.dir,
        idle: anims.idle,
        run: anims.run,
        run_backward: anims.run_backward,
        strafe_left: anims.strafe_left,
        strafe_right: anims.strafe_right,
        block: anims.block,
        hit_reaction: anims.hit_reaction,
        knockdown: anims.knockdown,
        getting_up: anims.getting_up,
        death: anims.death,
        sweep_fall: anims.sweep_fall,
        light_attack: moveset.light_attack.anim,
        heavy_attack: moveset.heavy_attack.anim,
        special_attack: moveset.special_attack.anim,
        mid_kick: moveset.mid_kick.anim,
        low_kick: moveset.low_kick.anim,
        aerial: moveset.aerial.anim,
        combo_finisher: moveset.combo_finisher.anim,
        super_attack: moveset.super_attack.anim,
    }
}

fn vec3_snap(v: wv_core::types::Vec3) -> Vec3Snapshot {
    Vec3Snapshot {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

fn phase_str(phase: GamePhase) -> &'static str {
    match phase {
        GamePhase::FighterSelect => "FighterSelect",
        GamePhase::Countdown => "Countdown",
        GamePhase::Fighting => "Fighting",
        GamePhase::RoundOver => "RoundOver",
        GamePhase::MatchOver => "MatchOver",
    }
}

fn state_str(state: FighterState) -> String {
    format!("{}", state)
}

fn state_num(state: FighterState) -> u8 {
    match state {
        FighterState::Idle => 0,
        FighterState::Moving => 1,
        FighterState::Attacking => 2,
        FighterState::Blocking => 3,
        FighterState::Dashing => 4,
        FighterState::HitStun => 5,
        FighterState::Airborne => 6,
        FighterState::Knockdown => 7,
        FighterState::GettingUp => 8,
    }
}

fn attack_type_str(a: ActiveAttack) -> &'static str {
    match a {
        ActiveAttack::Light => "Light",
        ActiveAttack::Heavy => "Heavy",
        ActiveAttack::Special => "Special",
        ActiveAttack::MidKick => "MidKick",
        ActiveAttack::LowKick => "LowKick",
        ActiveAttack::Aerial => "Aerial",
        ActiveAttack::ComboFinisher => "ComboFinisher",
        ActiveAttack::Super => "Super",
    }
}

fn attack_phase_str(p: AttackPhase) -> &'static str {
    match p {
        AttackPhase::Startup => "Startup",
        AttackPhase::Active => "Active",
        AttackPhase::Recovery => "Recovery",
    }
}

fn attack_phase_num(p: AttackPhase) -> u8 {
    match p {
        AttackPhase::Startup => 0,
        AttackPhase::Active => 1,
        AttackPhase::Recovery => 2,
    }
}

fn facing_str(f: wv_core::types::Facing) -> &'static str {
    match f {
        wv_core::types::Facing::Right => "Right",
        wv_core::types::Facing::Left => "Left",
    }
}

fn fighter_snap(f: &wv_core::game::Fighter) -> FighterSnapshot {
    let attack = f
        .state_machine
        .attack
        .map(|a| {
            let phase = f.state_machine.attack_phase.unwrap_or(AttackPhase::Startup);
            AttackSnapshot {
                attack_type: attack_type_str(a),
                phase: attack_phase_str(phase),
                phase_num: attack_phase_num(phase),
                hit_connected: f.state_machine.hit_connected,
                frame_counter: f.state_machine.frame_counter,
                total_frames: f.state_machine.total_frames,
            }
        });

    FighterSnapshot {
        fighter_id: format!("{}", f.data.id),
        weapon_type: format!("{}", f.weapon.weapon_type),
        position: vec3_snap(f.physics.position),
        velocity: vec3_snap(f.physics.velocity),
        health: f.health,
        max_health: f.data.max_health,
        health_pct: f.health_pct(),
        stamina: f.stamina,
        max_stamina: f.data.max_stamina,
        stamina_pct: f.stamina_pct(),
        facing: facing_str(f.facing),
        state: state_str(f.state_machine.state),
        state_num: state_num(f.state_machine.state),
        attack,
        round_wins: f.round_wins,
        grounded: f.physics.grounded,
        anim_dir: f.data.animations.dir,
        current_anim: f.current_animation().to_string(),
    }
}

pub fn snapshot(game: &GameState) -> GameSnapshot {
    GameSnapshot {
        phase: phase_str(game.phase),
        frame: game.frame,
        round_timer: game.round_time_remaining(),
        current_round: game.current_round,
        countdown_display: game.countdown_display(),
        last_hit_info: game.last_hit_info.clone(),
        winner: game.winner().map(|w| w as u8),
        fighters: [
            fighter_snap(&game.fighters[0]),
            fighter_snap(&game.fighters[1]),
        ],
    }
}
