use crate::combat;
use crate::fighter::{AttackData, FighterData, FighterId};
use crate::input::{ComboType, InputBuffer, InputState};
use crate::physics::PhysicsBody;
use crate::state_machine::{ActiveAttack, FighterState, StateMachine};
use crate::types::{Facing, Vec3};
use crate::weapon::WeaponData;

const DT: f32 = 1.0 / 60.0;
const ROUND_TIME_SECONDS: f32 = 60.0;
const ROUND_TIME_FRAMES: u32 = (ROUND_TIME_SECONDS * 60.0) as u32;
const ROUNDS_TO_WIN: u32 = 2;
const STAMINA_REGEN_RATE: f32 = 0.3; // per frame
const DASH_STAMINA_COST: f32 = 20.0;
const SPECIAL_STAMINA_COST: f32 = 30.0;
const AERIAL_STAMINA_COST: f32 = 15.0;
const SUPER_STAMINA_COST: f32 = 50.0;
const ATTACK_LUNGE: f32 = 3.5; // forward impulse when starting any attack

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    FighterSelect,
    Countdown,  // 3-2-1-FIGHT
    Fighting,
    RoundOver,
    MatchOver,
}

#[derive(Debug, Clone)]
pub struct Fighter {
    pub data: &'static FighterData,
    pub weapon: &'static WeaponData,
    pub state_machine: StateMachine,
    pub physics: PhysicsBody,
    pub input_buffer: InputBuffer,
    pub health: f32,
    pub stamina: f32,
    pub facing: Facing,
    pub round_wins: u32,
}

impl Fighter {
    pub fn new(id: FighterId, position: Vec3, facing: Facing) -> Self {
        let data = FighterData::get(id);
        let weapon = WeaponData::get(data.default_weapon);
        Self {
            data,
            weapon,
            state_machine: StateMachine::new(),
            physics: PhysicsBody::new(position),
            input_buffer: InputBuffer::new(),
            health: data.max_health,
            stamina: data.max_stamina,
            facing,
            round_wins: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn health_pct(&self) -> f32 {
        self.health / self.data.max_health
    }

    pub fn stamina_pct(&self) -> f32 {
        self.stamina / self.data.max_stamina
    }

    fn get_attack_data(&self, attack: ActiveAttack) -> &AttackData {
        match attack {
            ActiveAttack::Light => &self.data.moveset.light_attack,
            ActiveAttack::Heavy => &self.data.moveset.heavy_attack,
            ActiveAttack::Special => &self.data.moveset.special_attack,
            ActiveAttack::MidKick => &self.data.moveset.mid_kick,
            ActiveAttack::LowKick => &self.data.moveset.low_kick,
            ActiveAttack::Aerial => &self.data.moveset.aerial,
            ActiveAttack::ComboFinisher => &self.data.moveset.combo_finisher,
            ActiveAttack::Super => &self.data.moveset.super_attack,
        }
    }

    /// Returns the animation name the frontend should play for the current state.
    pub fn current_animation(&self) -> &str {
        // KO: always show death animation
        if self.health <= 0.0 {
            return self.data.animations.death;
        }

        match self.state_machine.state {
            FighterState::Idle => self.data.animations.idle,
            FighterState::Moving => {
                let fs = self.facing.sign();
                let fv = self.physics.velocity.x * fs; // positive = forward
                let lv = self.physics.velocity.z;
                if fv < -0.1 {
                    // Moving backward
                    self.data.animations.run_backward
                } else if lv.abs() > 0.1 && fv < 0.5 {
                    // Strafing — flip based on facing so model anim matches screen direction
                    let model_left = (lv < 0.0) == (self.facing == Facing::Right);
                    if model_left {
                        self.data.animations.strafe_left
                    } else {
                        self.data.animations.strafe_right
                    }
                } else {
                    self.data.animations.run
                }
            }
            FighterState::Attacking => {
                if let Some(attack) = self.state_machine.attack {
                    self.get_attack_data(attack).anim
                } else {
                    self.data.animations.idle
                }
            }
            FighterState::Blocking => self.data.animations.block,
            FighterState::Dashing => self.data.animations.run,
            FighterState::HitStun => self.data.animations.hit_reaction,
            FighterState::Airborne => self.data.animations.hit_reaction,
            FighterState::Knockdown => self.data.animations.knockdown,
            FighterState::GettingUp => self.data.animations.getting_up,
        }
    }

    /// Reset fighter state for a new round (keep round_wins).
    pub fn reset_round(&mut self, position: Vec3, facing: Facing) {
        self.state_machine = StateMachine::new();
        self.physics = PhysicsBody::new(position);
        self.input_buffer = InputBuffer::new();
        self.health = self.data.max_health;
        self.stamina = self.data.max_stamina;
        self.facing = facing;
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub fighters: [Fighter; 2],
    pub phase: GamePhase,
    pub frame: u32,
    pub round_timer: u32,
    pub current_round: u32,
    pub countdown_timer: u32,
    pub round_over_timer: u32,
    pub last_hit_info: Option<String>,
}

impl GameState {
    pub fn new(p1_fighter: FighterId, p2_fighter: FighterId) -> Self {
        Self {
            fighters: [
                Fighter::new(p1_fighter, Vec3::new(-1.5, 0.0, 0.0), Facing::Right),
                Fighter::new(p2_fighter, Vec3::new(1.5, 0.0, 0.0), Facing::Left),
            ],
            phase: GamePhase::Countdown,
            frame: 0,
            round_timer: ROUND_TIME_FRAMES,
            current_round: 1,
            countdown_timer: 180, // 3 seconds at 60fps
            round_over_timer: 0,
            last_hit_info: None,
        }
    }

    pub fn new_in_select() -> Self {
        Self {
            fighters: [
                Fighter::new(FighterId::Kael, Vec3::new(-1.5, 0.0, 0.0), Facing::Right),
                Fighter::new(FighterId::Kael, Vec3::new(1.5, 0.0, 0.0), Facing::Left),
            ],
            phase: GamePhase::FighterSelect,
            frame: 0,
            round_timer: ROUND_TIME_FRAMES,
            current_round: 1,
            countdown_timer: 180,
            round_over_timer: 0,
            last_hit_info: None,
        }
    }

    /// Set fighters after selection and start countdown.
    pub fn select_fighters(&mut self, p1: FighterId, p2: FighterId) {
        self.fighters = [
            Fighter::new(p1, Vec3::new(-1.5, 0.0, 0.0), Facing::Right),
            Fighter::new(p2, Vec3::new(1.5, 0.0, 0.0), Facing::Left),
        ];
        self.phase = GamePhase::Countdown;
        self.countdown_timer = 180;
    }

    pub fn winner(&self) -> Option<usize> {
        if self.fighters[0].round_wins >= ROUNDS_TO_WIN {
            Some(0)
        } else if self.fighters[1].round_wins >= ROUNDS_TO_WIN {
            Some(1)
        } else {
            None
        }
    }

    pub fn countdown_display(&self) -> &'static str {
        if self.countdown_timer > 120 {
            "3"
        } else if self.countdown_timer > 60 {
            "2"
        } else if self.countdown_timer > 0 {
            "1"
        } else {
            "FIGHT!"
        }
    }

    /// Main game tick. Call once per frame (60fps).
    pub fn tick(&mut self, p1_input: &InputState, p2_input: &InputState) {
        self.frame += 1;

        match self.phase {
            GamePhase::FighterSelect => {
                // Handled externally by the CLI
            }

            GamePhase::Countdown => {
                if self.countdown_timer > 0 {
                    self.countdown_timer -= 1;
                } else {
                    self.phase = GamePhase::Fighting;
                }
            }

            GamePhase::Fighting => {
                self.tick_fighting(p1_input, p2_input);
            }

            GamePhase::RoundOver => {
                if self.round_over_timer > 0 {
                    self.round_over_timer -= 1;
                } else {
                    // Check for match over
                    if self.winner().is_some() {
                        self.phase = GamePhase::MatchOver;
                    } else {
                        // Start next round
                        self.current_round += 1;
                        self.fighters[0].reset_round(
                            Vec3::new(-1.5, 0.0, 0.0),
                            Facing::Right,
                        );
                        self.fighters[1].reset_round(
                            Vec3::new(1.5, 0.0, 0.0),
                            Facing::Left,
                        );
                        self.round_timer = ROUND_TIME_FRAMES;
                        self.countdown_timer = 180;
                        self.phase = GamePhase::Countdown;
                        self.last_hit_info = None;
                    }
                }
            }

            GamePhase::MatchOver => {
                // Game is done, nothing to tick
            }
        }
    }

    fn tick_fighting(&mut self, p1_input: &InputState, p2_input: &InputState) {
        let inputs = [p1_input.clone(), p2_input.clone()];

        // Update input buffers
        for i in 0..2 {
            self.fighters[i].input_buffer.set_frame(self.frame);
            self.fighters[i].input_buffer.expire_old();
        }

        // Process inputs for each fighter
        for i in 0..2 {
            self.process_input(i, &inputs[i]);
        }

        // Apply forward lunge when an attack just started
        for i in 0..2 {
            if self.fighters[i].state_machine.state == FighterState::Attacking
                && self.fighters[i].state_machine.frame_counter == 0
            {
                let lunge = self.fighters[i].facing.sign() * ATTACK_LUNGE;
                self.fighters[i].physics.knockback.x = lunge;
            }
        }

        // Update state machines
        for fighter in &mut self.fighters {
            fighter.state_machine.tick();
        }

        // Regenerate stamina
        for fighter in &mut self.fighters {
            if fighter.state_machine.can_act() {
                fighter.stamina = (fighter.stamina + STAMINA_REGEN_RATE)
                    .min(fighter.data.max_stamina);
            }
        }

        // Align fighters on Z-axis — keep them on the same plane so attacks
        // always connect. Both fighters lerp toward their midpoint Z each frame.
        let mid_z = (self.fighters[0].physics.position.z
            + self.fighters[1].physics.position.z)
            * 0.5;
        for fighter in &mut self.fighters {
            fighter.physics.position.z += (mid_z - fighter.physics.position.z) * 0.3;
        }

        // Update facing (face opponent)
        let p0x = self.fighters[0].physics.position.x;
        let p1x = self.fighters[1].physics.position.x;
        if self.fighters[0].state_machine.can_act() {
            self.fighters[0].facing = if p0x < p1x {
                Facing::Right
            } else {
                Facing::Left
            };
        }
        if self.fighters[1].state_machine.can_act() {
            self.fighters[1].facing = if p1x < p0x {
                Facing::Right
            } else {
                Facing::Left
            };
        }

        // Check combat hits
        self.check_combat();

        // Update physics
        for fighter in &mut self.fighters {
            let landed = fighter.physics.tick(DT);
            if landed && fighter.state_machine.state == FighterState::Airborne {
                fighter.state_machine.land();
                fighter.physics.stop_movement();
            }
        }

        // Timer
        if self.round_timer > 0 {
            self.round_timer -= 1;
        }

        // Check round end conditions
        self.check_round_end();
    }

    fn process_input(&mut self, idx: usize, input: &InputState) {
        let fighter = &mut self.fighters[idx];

        if !fighter.state_machine.can_act() {
            // Can only release block
            if fighter.state_machine.state == FighterState::Blocking && !input.block {
                fighter.state_machine.stop_block();
            }
            return;
        }

        // Block
        if input.block {
            fighter.state_machine.start_block();
            fighter.physics.stop_movement();
            return;
        }

        // Dash
        if input.dash && fighter.stamina >= DASH_STAMINA_COST {
            if fighter.state_machine.start_dash(fighter.data.dash_frames) {
                fighter.stamina -= DASH_STAMINA_COST;
                let dash_vel = Vec3::new(
                    fighter.facing.sign() * fighter.data.dash_speed,
                    0.0,
                    0.0,
                );
                fighter.physics.set_movement(dash_vel);
                return;
            }
        }

        // Check for combos before processing individual attacks
        if input.light_attack {
            fighter.input_buffer.push(crate::input::InputAction::LightAttack);
        }
        if input.heavy_attack {
            fighter.input_buffer.push(crate::input::InputAction::HeavyAttack);
        }
        if input.special {
            fighter.input_buffer.push(crate::input::InputAction::Special);
        }

        if let Some(combo) = fighter.input_buffer.detect_combo() {
            let attack_type = match combo {
                ComboType::ThreeHit => ActiveAttack::ComboFinisher,
                ComboType::Super => ActiveAttack::Super,
            };
            let attack_data = fighter.get_attack_data(attack_type);
            let can_afford = match combo {
                ComboType::Super => fighter.stamina >= SUPER_STAMINA_COST,
                _ => true,
            };

            if can_afford {
                let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
                let active = attack_data.active_frames;
                let recovery = attack_data.recovery_frames;

                if fighter.state_machine.start_attack(attack_type, startup, active, recovery) {
                    fighter.input_buffer.clear();
                    fighter.physics.stop_movement();
                    if matches!(combo, ComboType::Super) {
                        fighter.stamina -= SUPER_STAMINA_COST;
                    }
                    return;
                }
            }
        }

        // Individual attacks
        if input.light_attack {
            let attack_data = fighter.get_attack_data(ActiveAttack::Light);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::Light,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.physics.stop_movement();
                return;
            }
        }

        if input.heavy_attack {
            let attack_data = fighter.get_attack_data(ActiveAttack::Heavy);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::Heavy,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.physics.stop_movement();
                return;
            }
        }

        if input.special && fighter.stamina >= SPECIAL_STAMINA_COST {
            let attack_data = fighter.get_attack_data(ActiveAttack::Special);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::Special,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.stamina -= SPECIAL_STAMINA_COST;
                fighter.physics.stop_movement();
                return;
            }
        }

        if input.mid_kick {
            let attack_data = fighter.get_attack_data(ActiveAttack::MidKick);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::MidKick,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.physics.stop_movement();
                return;
            }
        }

        if input.low_kick {
            let attack_data = fighter.get_attack_data(ActiveAttack::LowKick);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::LowKick,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.physics.stop_movement();
                return;
            }
        }

        if input.aerial && fighter.stamina >= AERIAL_STAMINA_COST {
            let attack_data = fighter.get_attack_data(ActiveAttack::Aerial);
            let startup = (attack_data.startup_frames as f32 / fighter.weapon.attack_speed) as u32;
            if fighter.state_machine.start_attack(
                ActiveAttack::Aerial,
                startup,
                attack_data.active_frames,
                attack_data.recovery_frames,
            ) {
                fighter.stamina -= AERIAL_STAMINA_COST;
                fighter.physics.stop_movement();
                return;
            }
        }

        // Movement
        if input.has_movement() {
            let mut vel = Vec3::ZERO;
            let speed = fighter.data.move_speed;
            if input.move_forward {
                vel.x += fighter.facing.sign() * speed;
            }
            if input.move_back {
                vel.x -= fighter.facing.sign() * speed;
            }
            if input.move_left {
                vel.z -= speed * 0.5;
            }
            if input.move_right {
                vel.z += speed * 0.5;
            }
            fighter.physics.set_movement(vel);
            fighter.state_machine.set_moving();
        } else {
            fighter.physics.stop_movement();
            fighter.state_machine.set_idle();
        }
    }

    fn check_combat(&mut self) {
        // Check each fighter's attack against the other
        for attacker_idx in 0..2 {
            let defender_idx = 1 - attacker_idx;

            if !self.fighters[attacker_idx].state_machine.is_attack_active() {
                continue;
            }
            if self.fighters[attacker_idx].state_machine.hit_connected {
                continue;
            }
            if !self.fighters[defender_idx].state_machine.is_vulnerable() {
                continue;
            }

            let attack_type = match self.fighters[attacker_idx].state_machine.attack {
                Some(a) => a,
                None => continue,
            };

            // Copy all data we need before mutating
            let attack_data = self.fighters[attacker_idx].get_attack_data(attack_type).clone();
            let attacker_pos = self.fighters[attacker_idx].physics.position;
            let attacker_facing = self.fighters[attacker_idx].facing;
            let attacker_weapon = self.fighters[attacker_idx].weapon;
            let attacker_defense = self.fighters[attacker_idx].data.defense;
            let defender_pos = self.fighters[defender_idx].physics.position;
            let defender_hurtbox = self.fighters[defender_idx].data.hurtbox;
            let defender_defense = self.fighters[defender_idx].data.defense;
            let is_blocking =
                self.fighters[defender_idx].state_machine.state == FighterState::Blocking;

            let hit_result = combat::check_hit(
                attacker_pos,
                attacker_facing,
                &attack_data,
                attacker_weapon,
                attacker_defense,
                defender_pos,
                &defender_hurtbox,
                defender_defense,
                is_blocking,
            );

            if let Some(hit) = hit_result {
                // Apply damage
                self.fighters[defender_idx].health =
                    (self.fighters[defender_idx].health - hit.damage).max(0.0);

                // Apply knockback
                self.fighters[defender_idx]
                    .physics
                    .apply_knockback(hit.knockback);

                // Apply state change based on hit severity
                let is_ko = self.fighters[defender_idx].health <= 0.0;
                let causes_knockdown = matches!(
                    attack_type,
                    ActiveAttack::ComboFinisher | ActiveAttack::Super | ActiveAttack::LowKick
                );

                if is_ko {
                    // KO: enter knockdown and stay down (very long timer)
                    self.fighters[defender_idx]
                        .state_machine
                        .enter_knockdown(9999);
                } else if hit.launches {
                    self.fighters[defender_idx].state_machine.enter_airborne();
                } else if !is_blocking && causes_knockdown {
                    // Hard knockdown: fall down, then get up
                    self.fighters[defender_idx]
                        .state_machine
                        .enter_knockdown(40);
                } else if !is_blocking {
                    self.fighters[defender_idx]
                        .state_machine
                        .enter_hitstun(hit.hitstun_frames);
                }

                // Mark hit as connected
                self.fighters[attacker_idx].state_machine.hit_connected = true;

                // Record hit info
                self.last_hit_info = Some(format!(
                    "P{} {} -> P{} for {:.1} dmg{}",
                    attacker_idx + 1,
                    attack_data.name,
                    defender_idx + 1,
                    hit.damage,
                    if hit.was_blocked { " (BLOCKED)" } else { "" },
                ));
            }
        }
    }

    fn check_round_end(&mut self) {
        let p1_dead = !self.fighters[0].is_alive();
        let p2_dead = !self.fighters[1].is_alive();
        let time_up = self.round_timer == 0;

        if p1_dead || p2_dead || time_up {
            // Determine round winner
            if p1_dead && !p2_dead {
                self.fighters[1].round_wins += 1;
            } else if p2_dead && !p1_dead {
                self.fighters[0].round_wins += 1;
            } else if time_up {
                // Higher health percentage wins
                if self.fighters[0].health_pct() > self.fighters[1].health_pct() {
                    self.fighters[0].round_wins += 1;
                } else if self.fighters[1].health_pct() > self.fighters[0].health_pct() {
                    self.fighters[1].round_wins += 1;
                }
                // Draw: no one gets a point
            }

            self.phase = GamePhase::RoundOver;
            self.round_over_timer = 120; // 2 seconds pause
        }
    }

    pub fn round_time_remaining(&self) -> f32 {
        self.round_timer as f32 / 60.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_input() -> InputState {
        InputState::default()
    }

    #[test]
    fn game_starts_in_countdown() {
        let game = GameState::new(FighterId::Kael, FighterId::Knight);
        assert_eq!(game.phase, GamePhase::Countdown);
    }

    #[test]
    fn countdown_transitions_to_fighting() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Knight);
        let input = empty_input();
        for _ in 0..200 {
            game.tick(&input, &input);
        }
        assert_eq!(game.phase, GamePhase::Fighting);
    }

    #[test]
    fn fighter_takes_damage() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Kael);
        game.phase = GamePhase::Fighting;

        // Move fighters close
        game.fighters[0].physics.position = Vec3::new(0.0, 0.0, 0.0);
        game.fighters[1].physics.position = Vec3::new(1.5, 0.0, 0.0);

        let mut attack_input = empty_input();
        attack_input.light_attack = true;

        let initial_health = game.fighters[1].health;

        // Tick enough frames for attack to hit
        game.tick(&attack_input, &empty_input());
        let normal_input = empty_input();
        for _ in 0..20 {
            game.tick(&normal_input, &normal_input);
        }

        assert!(game.fighters[1].health < initial_health);
    }

    #[test]
    fn blocking_reduces_damage() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Kael);
        game.phase = GamePhase::Fighting;

        game.fighters[0].physics.position = Vec3::new(0.0, 0.0, 0.0);
        game.fighters[1].physics.position = Vec3::new(1.5, 0.0, 0.0);

        let mut attack_input = empty_input();
        attack_input.light_attack = true;

        let mut block_input = empty_input();
        block_input.block = true;

        // Attack while blocking
        game.tick(&attack_input, &block_input);
        let normal_input = empty_input();
        for _ in 0..20 {
            game.tick(&normal_input, &block_input);
        }

        let blocked_health = game.fighters[1].health;

        // Reset for unblocked
        let mut game2 = GameState::new(FighterId::Kael, FighterId::Kael);
        game2.phase = GamePhase::Fighting;
        game2.fighters[0].physics.position = Vec3::new(0.0, 0.0, 0.0);
        game2.fighters[1].physics.position = Vec3::new(1.5, 0.0, 0.0);

        game2.tick(&attack_input, &empty_input());
        for _ in 0..20 {
            game2.tick(&normal_input, &normal_input);
        }

        let unblocked_health = game2.fighters[1].health;

        // Blocked should take less damage (more health remaining)
        assert!(blocked_health > unblocked_health);
    }

    #[test]
    fn round_ends_on_ko() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Knight);
        game.phase = GamePhase::Fighting;
        game.fighters[1].health = 0.0;

        game.tick(&empty_input(), &empty_input());
        assert_eq!(game.phase, GamePhase::RoundOver);
        assert_eq!(game.fighters[0].round_wins, 1);
    }

    #[test]
    fn round_ends_on_timer() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Knight);
        game.phase = GamePhase::Fighting;
        game.round_timer = 1;
        game.fighters[0].health = 80.0;
        game.fighters[1].health = 50.0;

        game.tick(&empty_input(), &empty_input());
        assert_eq!(game.phase, GamePhase::RoundOver);
        // P1 has more health, should win
        assert_eq!(game.fighters[0].round_wins, 1);
    }

    #[test]
    fn match_ends_after_enough_round_wins() {
        let mut game = GameState::new(FighterId::Kael, FighterId::Knight);
        game.phase = GamePhase::Fighting;
        game.fighters[0].round_wins = 1; // Already won 1
        game.fighters[1].health = 0.0;

        game.tick(&empty_input(), &empty_input());
        assert_eq!(game.phase, GamePhase::RoundOver);
        assert_eq!(game.fighters[0].round_wins, 2);

        // Tick through round over
        for _ in 0..130 {
            game.tick(&empty_input(), &empty_input());
        }
        assert_eq!(game.phase, GamePhase::MatchOver);
    }
}
