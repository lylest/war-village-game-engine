/// The 9 possible states a fighter can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FighterState {
    Idle,
    Moving,
    Attacking,
    Blocking,
    Dashing,
    HitStun,
    Airborne,
    Knockdown,
    GettingUp,
}

/// Which attack is being performed (used when state == Attacking).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackPhase {
    Startup,
    Active,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveAttack {
    Light,
    Heavy,
    Special,
    MidKick,
    LowKick,
    Aerial,
    ComboFinisher,
    Super,
}

#[derive(Debug, Clone)]
pub struct StateMachine {
    pub state: FighterState,
    pub frame_counter: u32,
    pub total_frames: u32,
    pub attack: Option<ActiveAttack>,
    pub attack_phase: Option<AttackPhase>,
    pub attack_startup: u32,
    pub attack_active: u32,
    pub attack_recovery: u32,
    pub hit_connected: bool, // prevents multi-hit per attack
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: FighterState::Idle,
            frame_counter: 0,
            total_frames: 0,
            attack: None,
            attack_phase: None,
            attack_startup: 0,
            attack_active: 0,
            attack_recovery: 0,
            hit_connected: false,
        }
    }

    /// Whether the fighter can act (accept new commands).
    pub fn can_act(&self) -> bool {
        matches!(self.state, FighterState::Idle | FighterState::Moving)
    }

    /// Whether the fighter can block right now.
    pub fn can_block(&self) -> bool {
        self.can_act()
    }

    /// Whether the fighter is in a state where they can be hit.
    pub fn is_vulnerable(&self) -> bool {
        !matches!(
            self.state,
            FighterState::Dashing | FighterState::GettingUp | FighterState::Knockdown
        )
    }

    /// Whether the fighter is currently in the active frames of an attack.
    pub fn is_attack_active(&self) -> bool {
        self.state == FighterState::Attacking
            && self.attack_phase == Some(AttackPhase::Active)
    }

    /// Start an attack. Returns false if the fighter can't attack right now.
    pub fn start_attack(
        &mut self,
        attack: ActiveAttack,
        startup: u32,
        active: u32,
        recovery: u32,
    ) -> bool {
        if !self.can_act() {
            return false;
        }
        self.state = FighterState::Attacking;
        self.attack = Some(attack);
        self.attack_phase = Some(AttackPhase::Startup);
        self.attack_startup = startup;
        self.attack_active = active;
        self.attack_recovery = recovery;
        self.total_frames = startup + active + recovery;
        self.frame_counter = 0;
        self.hit_connected = false;
        true
    }

    /// Start blocking.
    pub fn start_block(&mut self) -> bool {
        if !self.can_block() {
            return false;
        }
        self.state = FighterState::Blocking;
        self.frame_counter = 0;
        self.total_frames = 0; // blocking persists until released
        true
    }

    /// Stop blocking.
    pub fn stop_block(&mut self) {
        if self.state == FighterState::Blocking {
            self.state = FighterState::Idle;
            self.frame_counter = 0;
        }
    }

    /// Start a dash.
    pub fn start_dash(&mut self, dash_frames: u32) -> bool {
        if !self.can_act() {
            return false;
        }
        self.state = FighterState::Dashing;
        self.frame_counter = 0;
        self.total_frames = dash_frames;
        true
    }

    /// Enter hitstun (from being hit).
    pub fn enter_hitstun(&mut self, stun_frames: u32) {
        self.state = FighterState::HitStun;
        self.frame_counter = 0;
        self.total_frames = stun_frames;
        self.attack = None;
        self.attack_phase = None;
    }

    /// Enter airborne state (launched).
    pub fn enter_airborne(&mut self) {
        self.state = FighterState::Airborne;
        self.frame_counter = 0;
        self.total_frames = 0; // ends when landing
        self.attack = None;
        self.attack_phase = None;
    }

    /// Enter knockdown (from landing while airborne or hard knockdown).
    pub fn enter_knockdown(&mut self, down_frames: u32) {
        self.state = FighterState::Knockdown;
        self.frame_counter = 0;
        self.total_frames = down_frames;
    }

    /// Set moving state.
    pub fn set_moving(&mut self) {
        if self.can_act() {
            self.state = FighterState::Moving;
        }
    }

    /// Set idle state.
    pub fn set_idle(&mut self) {
        if self.state == FighterState::Moving {
            self.state = FighterState::Idle;
        }
    }

    /// Advance the state machine by one frame. Returns true if state changed.
    pub fn tick(&mut self) -> bool {
        match self.state {
            FighterState::Idle | FighterState::Moving | FighterState::Blocking => false,

            FighterState::Attacking => {
                self.frame_counter += 1;
                if self.frame_counter <= self.attack_startup {
                    if self.attack_phase != Some(AttackPhase::Startup) {
                        self.attack_phase = Some(AttackPhase::Startup);
                        return true;
                    }
                } else if self.frame_counter <= self.attack_startup + self.attack_active {
                    if self.attack_phase != Some(AttackPhase::Active) {
                        self.attack_phase = Some(AttackPhase::Active);
                        return true;
                    }
                } else if self.frame_counter
                    <= self.attack_startup + self.attack_active + self.attack_recovery
                {
                    if self.attack_phase != Some(AttackPhase::Recovery) {
                        self.attack_phase = Some(AttackPhase::Recovery);
                        return true;
                    }
                }

                if self.frame_counter >= self.total_frames {
                    self.state = FighterState::Idle;
                    self.attack = None;
                    self.attack_phase = None;
                    self.frame_counter = 0;
                    return true;
                }
                false
            }

            FighterState::Dashing => {
                self.frame_counter += 1;
                if self.frame_counter >= self.total_frames {
                    self.state = FighterState::Idle;
                    self.frame_counter = 0;
                    return true;
                }
                false
            }

            FighterState::HitStun => {
                self.frame_counter += 1;
                if self.frame_counter >= self.total_frames {
                    self.state = FighterState::Idle;
                    self.frame_counter = 0;
                    return true;
                }
                false
            }

            FighterState::Airborne => {
                self.frame_counter += 1;
                false
            }

            FighterState::Knockdown => {
                self.frame_counter += 1;
                if self.frame_counter >= self.total_frames {
                    self.state = FighterState::GettingUp;
                    self.frame_counter = 0;
                    self.total_frames = 20;
                    return true;
                }
                false
            }

            FighterState::GettingUp => {
                self.frame_counter += 1;
                if self.frame_counter >= self.total_frames {
                    self.state = FighterState::Idle;
                    self.frame_counter = 0;
                    return true;
                }
                false
            }
        }
    }

    /// Called when physics detects a landing while airborne.
    pub fn land(&mut self) {
        if self.state == FighterState::Airborne {
            self.enter_knockdown(30);
        }
    }
}

impl std::fmt::Display for FighterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FighterState::Idle => write!(f, "Idle"),
            FighterState::Moving => write!(f, "Moving"),
            FighterState::Attacking => write!(f, "Attacking"),
            FighterState::Blocking => write!(f, "Blocking"),
            FighterState::Dashing => write!(f, "Dashing"),
            FighterState::HitStun => write!(f, "HitStun"),
            FighterState::Airborne => write!(f, "Airborne"),
            FighterState::Knockdown => write!(f, "Knockdown"),
            FighterState::GettingUp => write!(f, "Getting Up"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_can_act() {
        let sm = StateMachine::new();
        assert!(sm.can_act());
        assert_eq!(sm.state, FighterState::Idle);
    }

    #[test]
    fn attack_transitions() {
        let mut sm = StateMachine::new();
        assert!(sm.start_attack(ActiveAttack::Light, 4, 3, 6));
        assert_eq!(sm.state, FighterState::Attacking);
        assert!(!sm.can_act());

        for _ in 0..3 {
            sm.tick();
        }
        assert_eq!(sm.attack_phase, Some(AttackPhase::Startup));

        sm.tick();
        assert_eq!(sm.attack_phase, Some(AttackPhase::Startup));
        sm.tick();
        assert_eq!(sm.attack_phase, Some(AttackPhase::Active));

        sm.tick();
        sm.tick();
        assert_eq!(sm.attack_phase, Some(AttackPhase::Active));

        sm.tick();
        assert_eq!(sm.attack_phase, Some(AttackPhase::Recovery));

        for _ in 0..5 {
            sm.tick();
        }
        assert_eq!(sm.state, FighterState::Idle);
    }

    #[test]
    fn dash_transitions() {
        let mut sm = StateMachine::new();
        assert!(sm.start_dash(10));
        assert_eq!(sm.state, FighterState::Dashing);

        for _ in 0..10 {
            sm.tick();
        }
        assert_eq!(sm.state, FighterState::Idle);
    }

    #[test]
    fn hitstun_to_idle() {
        let mut sm = StateMachine::new();
        sm.enter_hitstun(15);
        assert_eq!(sm.state, FighterState::HitStun);

        for _ in 0..15 {
            sm.tick();
        }
        assert_eq!(sm.state, FighterState::Idle);
    }

    #[test]
    fn knockdown_to_getup_to_idle() {
        let mut sm = StateMachine::new();
        sm.enter_knockdown(30);
        assert_eq!(sm.state, FighterState::Knockdown);

        for _ in 0..30 {
            sm.tick();
        }
        assert_eq!(sm.state, FighterState::GettingUp);

        for _ in 0..20 {
            sm.tick();
        }
        assert_eq!(sm.state, FighterState::Idle);
    }

    #[test]
    fn airborne_land() {
        let mut sm = StateMachine::new();
        sm.enter_airborne();
        assert_eq!(sm.state, FighterState::Airborne);

        sm.land();
        assert_eq!(sm.state, FighterState::Knockdown);
    }

    #[test]
    fn cannot_act_during_attack() {
        let mut sm = StateMachine::new();
        sm.start_attack(ActiveAttack::Light, 4, 3, 6);
        assert!(!sm.start_attack(ActiveAttack::Heavy, 10, 4, 12));
        assert!(!sm.start_dash(10));
    }
}
