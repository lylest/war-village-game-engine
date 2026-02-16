use crate::types::Vec3;

const GRAVITY: f32 = -20.0; // units/s² (tuned for game feel, not realism)
const KNOCKBACK_DECAY: f32 = 0.85; // multiplier per frame
const KNOCKBACK_THRESHOLD: f32 = 0.1; // below this, knockback stops

// Arena bounds (x-axis is the fighting axis, z is depth)
pub const ARENA_MIN_X: f32 = -10.0;
pub const ARENA_MAX_X: f32 = 10.0;
pub const ARENA_MIN_Z: f32 = -3.0;
pub const ARENA_MAX_Z: f32 = 3.0;
pub const GROUND_Y: f32 = 0.0;

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub knockback: Vec3,
    pub grounded: bool,
}

impl PhysicsBody {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            knockback: Vec3::ZERO,
            grounded: true,
        }
    }

    /// Apply a knockback impulse.
    pub fn apply_knockback(&mut self, force: Vec3) {
        self.knockback = force;
        if force.y > 0.0 {
            self.grounded = false;
        }
    }

    /// Update physics for one frame. `dt` is the time step (1/60 for 60fps).
    /// Returns true if the body just landed (was airborne, now grounded).
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut just_landed = false;

        // Apply gravity before movement so it takes effect this frame
        if !self.grounded {
            self.velocity.y += GRAVITY * dt;
        }

        // Apply knockback decay
        self.knockback = self.knockback * KNOCKBACK_DECAY;
        if self.knockback.length() < KNOCKBACK_THRESHOLD {
            self.knockback = Vec3::ZERO;
        }

        // Combine velocity + knockback
        let total_vel = self.velocity + self.knockback;
        self.position += total_vel * dt;

        // Ground collision
        if !self.grounded && self.position.y <= GROUND_Y {
            self.position.y = GROUND_Y;
            self.velocity.y = 0.0;
            self.grounded = true;
            just_landed = true;
        }

        // Clamp to arena bounds
        self.position.x = self.position.x.clamp(ARENA_MIN_X, ARENA_MAX_X);
        self.position.z = self.position.z.clamp(ARENA_MIN_Z, ARENA_MAX_Z);

        // Keep on ground if grounded
        if self.grounded {
            self.position.y = GROUND_Y;
        }

        just_landed
    }

    /// Set movement velocity (from player input). Does not override knockback.
    pub fn set_movement(&mut self, vel: Vec3) {
        self.velocity.x = vel.x;
        self.velocity.z = vel.z;
        // Don't touch velocity.y — that's managed by gravity/jumping
    }

    /// Stop horizontal movement.
    pub fn stop_movement(&mut self) {
        self.velocity.x = 0.0;
        self.velocity.z = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_stays_on_ground() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 0.0, 0.0));
        assert!(body.grounded);
        body.tick(1.0 / 60.0);
        assert_eq!(body.position.y, GROUND_Y);
    }

    #[test]
    fn gravity_pulls_down() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 5.0, 0.0));
        body.grounded = false;
        body.tick(1.0 / 60.0);
        assert!(body.position.y < 5.0);
    }

    #[test]
    fn landing_detection() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 0.5, 0.0));
        body.grounded = false;
        body.velocity.y = -10.0;
        let landed = body.tick(1.0 / 60.0);
        // May or may not land in one frame depending on velocity
        // but after enough frames it should land
        if !landed {
            for _ in 0..100 {
                if body.tick(1.0 / 60.0) {
                    break;
                }
            }
        }
        assert!(body.grounded);
        assert_eq!(body.position.y, GROUND_Y);
    }

    #[test]
    fn arena_bounds_clamping() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 0.0, 0.0));
        body.velocity.x = 1000.0;
        body.tick(1.0 / 60.0);
        assert!(body.position.x <= ARENA_MAX_X);

        body.velocity.x = -1000.0;
        body.tick(1.0 / 60.0);
        assert!(body.position.x >= ARENA_MIN_X);
    }

    #[test]
    fn knockback_decays() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 0.0, 0.0));
        body.apply_knockback(Vec3::new(10.0, 0.0, 0.0));
        assert!(body.knockback.x > 0.0);

        for _ in 0..60 {
            body.tick(1.0 / 60.0);
        }
        // After many frames, knockback should have decayed to near zero
        assert!(body.knockback.length() < KNOCKBACK_THRESHOLD);
    }

    #[test]
    fn knockback_with_launch() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 0.0, 0.0));
        body.apply_knockback(Vec3::new(5.0, 8.0, 0.0));
        assert!(!body.grounded);

        body.tick(1.0 / 60.0);
        assert!(body.position.y > 0.0 || body.position.x != 0.0);
    }

    #[test]
    fn movement_does_not_override_y() {
        let mut body = PhysicsBody::new(Vec3::new(0.0, 5.0, 0.0));
        body.grounded = false;
        body.velocity.y = -5.0;
        body.set_movement(Vec3::new(3.0, 0.0, 0.0));
        assert_eq!(body.velocity.y, -5.0);
        assert_eq!(body.velocity.x, 3.0);
    }
}
