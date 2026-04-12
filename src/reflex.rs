use crate::types::InstinctType;

#[derive(Clone, Debug)]
pub struct Reflex {
    pub instinct: InstinctType,
    pub urgency: f32,
    pub energy_cost: f32,
    pub suppressed: bool,
}

impl Reflex {
    pub fn new(instinct: InstinctType, urgency: f32) -> Self {
        let urgency = urgency.clamp(0.0, 1.0);
        let energy_cost = match instinct {
            InstinctType::Survive => 0.3,
            InstinctType::Flee => 0.25,
            InstinctType::Guard => 0.1,
            InstinctType::Hoard => 0.05,
            InstinctType::Cooperate => 0.08,
            InstinctType::Teach => 0.12,
            InstinctType::Curious => 0.03,
            InstinctType::Mourn => 0.02,
            InstinctType::Evolve => 0.15,
            _ => 0.0,
        };
        Self { instinct, urgency, energy_cost, suppressed: false }
    }

    pub fn suppress(&mut self) {
        self.suppressed = true;
    }

    pub fn is_active(&self) -> bool {
        !self.suppressed
    }
}
