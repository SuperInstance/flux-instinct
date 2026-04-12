use crate::reflex::Reflex;
use crate::thresholds::Thresholds;
use crate::types::InstinctType;

pub struct InstinctEngine {
    thresholds: Thresholds,
    cycle_count: u64,
    idle_count: u64,
    peer_alive: bool,
    has_work: bool,
    last_reflexes: Vec<Reflex>,
}

impl InstinctEngine {
    pub fn new(thresholds: Thresholds) -> Self {
        Self {
            thresholds,
            cycle_count: 0,
            idle_count: 0,
            peer_alive: true,
            has_work: false,
            last_reflexes: Vec::new(),
        }
    }

    pub fn tick(&mut self, energy_frac: f32, threat: f32, trust: f32, peer_alive: bool, has_work: bool) -> &[Reflex] {
        self.cycle_count += 1;
        let peer_died = self.peer_alive && !peer_alive;
        self.peer_alive = peer_alive;
        self.has_work = has_work;

        // Track idle (no high-urgency reflexes fired)
        let had_work = has_work || energy_frac < self.thresholds.energy_critical || threat > self.thresholds.threat_high;
        if !had_work {
            self.idle_count += 1;
        } else {
            self.idle_count = 0;
        }

        let mut reflexes: Vec<Reflex> = Vec::new();

        // Survive: always fires if energy critical
        if energy_frac <= self.thresholds.energy_critical {
            reflexes.push(Reflex::new(InstinctType::Survive, 1.0));
        }

        // Flee: threat > threshold
        if threat > self.thresholds.threat_high {
            let urgency = ((threat - self.thresholds.threat_high) / (1.0 - self.thresholds.threat_high)).clamp(0.0, 1.0);
            reflexes.push(Reflex::new(InstinctType::Flee, urgency));
        }

        // Guard: has_work and energy ok
        if has_work && energy_frac > self.thresholds.energy_critical && self.thresholds.guard_has_work {
            reflexes.push(Reflex::new(InstinctType::Guard, 0.5));
        }

        // Cooperate: trust > threshold
        if trust > self.thresholds.trust_cooperate {
            reflexes.push(Reflex::new(InstinctType::Cooperate, 0.5));
        }

        // Teach: trust very high
        if trust > self.thresholds.trust_teach {
            reflexes.push(Reflex::new(InstinctType::Teach, 0.6));
        }

        // Curious: every idle_trigger cycles
        if self.idle_count > 0 && self.idle_count % self.thresholds.idle_trigger as u64 == 0 {
            reflexes.push(Reflex::new(InstinctType::Curious, 0.3));
        }

        // Evolve: every evolve_idle cycles
        if self.idle_count > 0 && self.idle_count % self.thresholds.evolve_idle as u64 == 0 {
            reflexes.push(Reflex::new(InstinctType::Evolve, 0.2));
        }

        // Hoard: energy below threshold but not critical
        if energy_frac <= self.thresholds.energy_below && energy_frac > self.thresholds.energy_critical {
            reflexes.push(Reflex::new(InstinctType::Hoard, 0.6));
        }

        // Mourn: peer just died
        if peer_died {
            reflexes.push(Reflex::new(InstinctType::Mourn, 0.8));
        }

        // Report: anomaly (we use report_anomaly flag as a simple toggle)
        if self.thresholds.report_anomaly {
            // Report fires when threat is elevated but below flee threshold (anomaly zone)
            if threat > 0.3 && threat <= self.thresholds.threat_high {
                reflexes.push(Reflex::new(InstinctType::Report, 0.4));
            }
        }

        // Sort by urgency descending, then by instinct priority ascending
        reflexes.sort_by(|a, b| {
            b.urgency.partial_cmp(&a.urgency).unwrap()
                .then_with(|| a.instinct.cmp(&b.instinct))
        });

        self.last_reflexes = reflexes;
        &self.last_reflexes
    }

    pub fn highest_priority(&self) -> Option<&Reflex> {
        self.last_reflexes.first()
    }

    pub fn is_firing(&self, t: InstinctType) -> bool {
        self.last_reflexes.iter().any(|r| r.instinct == t && r.is_active())
    }

    pub fn suppress(&mut self, t: InstinctType) {
        for r in &mut self.last_reflexes {
            if r.instinct == t {
                r.suppress();
            }
        }
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    pub fn idle_count(&self) -> u64 {
        self.idle_count
    }
}
