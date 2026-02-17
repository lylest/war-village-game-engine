mod snapshot;

use snapshot::{ArenaBounds, GameSnapshot};
use wasm_bindgen::prelude::*;
use wv_core::fighter::{FighterData, FighterId};
use wv_core::game::GameState;
use wv_core::input::InputState;
use wv_core::physics;

fn parse_fighter_id(name: &str) -> Result<FighterId, JsError> {
    match name.to_lowercase().as_str() {
        "kael" => Ok(FighterId::Kael),
        "knight" => Ok(FighterId::Knight),
        "zara" => Ok(FighterId::Zara),
        "magnus" => Ok(FighterId::Magnus),
        "orin" => Ok(FighterId::Orin),
        _ => Err(JsError::new(&format!(
            "Unknown fighter '{}'. Choose from: Kael, Knight, Zara, Magnus, Orin",
            name
        ))),
    }
}

fn to_js(snap: &GameSnapshot) -> JsValue {
    serde_wasm_bindgen::to_value(snap).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub struct WasmGame {
    state: GameState,
}

#[wasm_bindgen]
impl WasmGame {
    /// Create a new game with two fighters by name.
    #[wasm_bindgen(constructor)]
    pub fn new(p1: &str, p2: &str) -> Result<WasmGame, JsError> {
        let p1_id = parse_fighter_id(p1)?;
        let p2_id = parse_fighter_id(p2)?;
        Ok(WasmGame {
            state: GameState::new(p1_id, p2_id),
        })
    }

    /// Advance one frame with explicit boolean inputs.
    ///
    /// P1: move_fwd, move_back, move_left, move_right, light, heavy, special, block, dash, mid_kick, low_kick, aerial
    /// P2: same order
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        p1_fwd: bool, p1_back: bool, p1_left: bool, p1_right: bool,
        p1_light: bool, p1_heavy: bool, p1_special: bool,
        p1_block: bool, p1_dash: bool,
        p1_mid_kick: bool, p1_low_kick: bool, p1_aerial: bool,
        p2_fwd: bool, p2_back: bool, p2_left: bool, p2_right: bool,
        p2_light: bool, p2_heavy: bool, p2_special: bool,
        p2_block: bool, p2_dash: bool,
        p2_mid_kick: bool, p2_low_kick: bool, p2_aerial: bool,
    ) -> JsValue {
        let p1_input = InputState {
            move_forward: p1_fwd,
            move_back: p1_back,
            move_left: p1_left,
            move_right: p1_right,
            light_attack: p1_light,
            heavy_attack: p1_heavy,
            special: p1_special,
            block: p1_block,
            dash: p1_dash,
            mid_kick: p1_mid_kick,
            low_kick: p1_low_kick,
            aerial: p1_aerial,
        };
        let p2_input = InputState {
            move_forward: p2_fwd,
            move_back: p2_back,
            move_left: p2_left,
            move_right: p2_right,
            light_attack: p2_light,
            heavy_attack: p2_heavy,
            special: p2_special,
            block: p2_block,
            dash: p2_dash,
            mid_kick: p2_mid_kick,
            low_kick: p2_low_kick,
            aerial: p2_aerial,
        };
        self.state.tick(&p1_input, &p2_input);
        to_js(&snapshot::snapshot(&self.state))
    }

    /// Advance one frame with packed bitflag input.
    ///
    /// Bits 0-11 = P1 (fwd, back, left, right, light, heavy, special, block, dash, mid_kick, low_kick, aerial)
    /// Bits 12-23 = P2 (same order)
    pub fn tick_packed(&mut self, input: u32) -> JsValue {
        let p1_input = InputState {
            move_forward: input & (1 << 0) != 0,
            move_back: input & (1 << 1) != 0,
            move_left: input & (1 << 2) != 0,
            move_right: input & (1 << 3) != 0,
            light_attack: input & (1 << 4) != 0,
            heavy_attack: input & (1 << 5) != 0,
            special: input & (1 << 6) != 0,
            block: input & (1 << 7) != 0,
            dash: input & (1 << 8) != 0,
            mid_kick: input & (1 << 9) != 0,
            low_kick: input & (1 << 10) != 0,
            aerial: input & (1 << 11) != 0,
        };
        let p2_input = InputState {
            move_forward: input & (1 << 12) != 0,
            move_back: input & (1 << 13) != 0,
            move_left: input & (1 << 14) != 0,
            move_right: input & (1 << 15) != 0,
            light_attack: input & (1 << 16) != 0,
            heavy_attack: input & (1 << 17) != 0,
            special: input & (1 << 18) != 0,
            block: input & (1 << 19) != 0,
            dash: input & (1 << 20) != 0,
            mid_kick: input & (1 << 21) != 0,
            low_kick: input & (1 << 22) != 0,
            aerial: input & (1 << 23) != 0,
        };
        self.state.tick(&p1_input, &p2_input);
        to_js(&snapshot::snapshot(&self.state))
    }

    /// Get the current game snapshot without advancing a frame.
    pub fn get_snapshot(&self) -> JsValue {
        to_js(&snapshot::snapshot(&self.state))
    }

    /// Quick phase check (returns e.g. "Fighting", "Countdown", "MatchOver").
    pub fn phase(&self) -> String {
        match self.state.phase {
            wv_core::game::GamePhase::FighterSelect => "FighterSelect".into(),
            wv_core::game::GamePhase::Countdown => "Countdown".into(),
            wv_core::game::GamePhase::Fighting => "Fighting".into(),
            wv_core::game::GamePhase::RoundOver => "RoundOver".into(),
            wv_core::game::GamePhase::MatchOver => "MatchOver".into(),
        }
    }
}

/// Returns the list of available fighter names.
#[wasm_bindgen]
pub fn available_fighters() -> JsValue {
    let names: Vec<&str> = FighterId::ALL.iter().map(|id| match id {
        FighterId::Kael => "Kael",
        FighterId::Knight => "Knight",
        FighterId::Zara => "Zara",
        FighterId::Magnus => "Magnus",
        FighterId::Orin => "Orin",
    }).collect();
    serde_wasm_bindgen::to_value(&names).unwrap_or(JsValue::NULL)
}

/// Returns animation data for a fighter (dir, idle, run, block, etc.).
#[wasm_bindgen]
pub fn fighter_animations(name: &str) -> Result<JsValue, JsError> {
    let id = parse_fighter_id(name)?;
    let data = FighterData::get(id);
    let anims = snapshot::animation_set_snapshot(&data.animations, &data.moveset);
    serde_wasm_bindgen::to_value(&anims).map_err(|e| JsError::new(&e.to_string()))
}

/// Returns the arena bounds as { min_x, max_x, min_z, max_z }.
#[wasm_bindgen]
pub fn arena_bounds() -> JsValue {
    let bounds = ArenaBounds {
        min_x: physics::ARENA_MIN_X,
        max_x: physics::ARENA_MAX_X,
        min_z: physics::ARENA_MIN_Z,
        max_z: physics::ARENA_MAX_Z,
    };
    serde_wasm_bindgen::to_value(&bounds).unwrap_or(JsValue::NULL)
}
