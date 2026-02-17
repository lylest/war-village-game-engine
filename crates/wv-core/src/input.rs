use std::collections::VecDeque;

const MAX_BUFFER_SIZE: usize = 10;
const INPUT_EXPIRY_FRAMES: u32 = 60; // 1 second at 60fps

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
    LightAttack,
    HeavyAttack,
    Special,
    Block,
    Dash,
    MidKick,
    LowKick,
    Aerial,
}

#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub action: InputAction,
    pub frame: u32, // game frame when the input occurred
}

/// Detected combo from the input buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboType {
    ThreeHit,     // Light x3
    Super,        // Light, Heavy, Special
}

/// Per-player input state: which directions/buttons are held this frame.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub move_forward: bool,
    pub move_back: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub light_attack: bool,
    pub heavy_attack: bool,
    pub special: bool,
    pub block: bool,
    pub dash: bool,
    pub mid_kick: bool,
    pub low_kick: bool,
    pub aerial: bool,
}

impl InputState {
    pub fn has_movement(&self) -> bool {
        self.move_forward || self.move_back || self.move_left || self.move_right
    }
}

#[derive(Debug, Clone)]
pub struct InputBuffer {
    events: VecDeque<InputEvent>,
    current_frame: u32,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            current_frame: 0,
        }
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.current_frame = frame;
    }

    /// Record an input action.
    pub fn push(&mut self, action: InputAction) {
        // Only buffer attack inputs for combo detection
        if matches!(
            action,
            InputAction::LightAttack | InputAction::HeavyAttack | InputAction::Special
        ) {
            self.events.push_back(InputEvent {
                action,
                frame: self.current_frame,
            });
            if self.events.len() > MAX_BUFFER_SIZE {
                self.events.pop_front();
            }
        }
    }

    /// Remove expired inputs from the buffer.
    pub fn expire_old(&mut self) {
        while let Some(front) = self.events.front() {
            if self.current_frame.saturating_sub(front.frame) > INPUT_EXPIRY_FRAMES {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Check for combos in the buffer. Returns the highest-priority combo detected.
    pub fn detect_combo(&self) -> Option<ComboType> {
        let valid: Vec<_> = self
            .events
            .iter()
            .filter(|e| self.current_frame.saturating_sub(e.frame) <= INPUT_EXPIRY_FRAMES)
            .collect();

        if valid.len() < 3 {
            return None;
        }

        // Check last 3 inputs
        let last3 = &valid[valid.len() - 3..];

        // Light, Heavy, Special = Special combo
        if last3[0].action == InputAction::LightAttack
            && last3[1].action == InputAction::HeavyAttack
            && last3[2].action == InputAction::Special
        {
            return Some(ComboType::Super);
        }

        // Light x3 = ThreeHit combo
        if last3.iter().all(|e| e.action == InputAction::LightAttack) {
            return Some(ComboType::ThreeHit);
        }

        None
    }

    /// Clear the buffer (e.g., after a combo is consumed).
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_records_attacks() {
        let mut buf = InputBuffer::new();
        buf.push(InputAction::LightAttack);
        buf.push(InputAction::HeavyAttack);
        assert_eq!(buf.events.len(), 2);
    }

    #[test]
    fn buffer_ignores_movement() {
        let mut buf = InputBuffer::new();
        buf.push(InputAction::MoveForward);
        assert_eq!(buf.events.len(), 0);
    }

    #[test]
    fn detect_three_hit_combo() {
        let mut buf = InputBuffer::new();
        buf.set_frame(10);
        buf.push(InputAction::LightAttack);
        buf.set_frame(20);
        buf.push(InputAction::LightAttack);
        buf.set_frame(30);
        buf.push(InputAction::LightAttack);
        buf.set_frame(30);
        assert_eq!(buf.detect_combo(), Some(ComboType::ThreeHit));
    }

    #[test]
    fn detect_special_combo() {
        let mut buf = InputBuffer::new();
        buf.set_frame(10);
        buf.push(InputAction::LightAttack);
        buf.set_frame(20);
        buf.push(InputAction::HeavyAttack);
        buf.set_frame(30);
        buf.push(InputAction::Special);
        buf.set_frame(30);
        assert_eq!(buf.detect_combo(), Some(ComboType::Super));
    }

    #[test]
    fn expired_inputs_no_combo() {
        let mut buf = InputBuffer::new();
        buf.set_frame(0);
        buf.push(InputAction::LightAttack);
        buf.set_frame(10);
        buf.push(InputAction::LightAttack);
        buf.set_frame(200); // way past expiry
        buf.push(InputAction::LightAttack);
        buf.set_frame(200);
        assert_eq!(buf.detect_combo(), None);
    }

    #[test]
    fn buffer_max_size() {
        let mut buf = InputBuffer::new();
        for i in 0..15 {
            buf.set_frame(i);
            buf.push(InputAction::LightAttack);
        }
        assert_eq!(buf.events.len(), MAX_BUFFER_SIZE);
    }

    #[test]
    fn input_state_movement() {
        let mut state = InputState::default();
        assert!(!state.has_movement());
        state.move_forward = true;
        assert!(state.has_movement());
    }
}
