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

#[cfg(test)]
mod tests {
    use super::*;
use crate::history::{HistoryEntry, InstinctHistory};

    fn default_thresholds() -> Thresholds {
        Thresholds::default()
    }

    #[test]
    fn test_tick_survive_fires_at_critical_energy() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.1, 0.0, 0.5, true, false);
        assert!(reflexes.iter().any(|r| r.instinct == InstinctType::Survive));
    }

    #[test]
    fn test_tick_no_survive_at_normal_energy() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.8, 0.0, 0.5, true, false);
        assert!(!reflexes.iter().any(|r| r.instinct == InstinctType::Survive));
    }

    #[test]
    fn test_tick_flee_at_high_threat() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.5, 0.9, 0.5, true, false);
        assert!(reflexes.iter().any(|r| r.instinct == InstinctType::Flee));
    }

    #[test]
    fn test_tick_flee_urgency_scales_with_threat() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let r1 = engine.tick(0.5, 0.75, 0.5, true, false);
        let flee1_urgency = r1.iter().find(|r| r.instinct == InstinctType::Flee).unwrap().urgency;
        let r2 = engine.tick(0.5, 1.0, 0.5, true, false);
        let flee2_urgency = r2.iter().find(|r| r.instinct == InstinctType::Flee).unwrap().urgency;
        assert!(flee2_urgency > flee1_urgency);
    }

    #[test]
    fn test_tick_cooperate_at_high_trust() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.5, 0.0, 0.8, true, false);
        assert!(reflexes.iter().any(|r| r.instinct == InstinctType::Cooperate));
    }

    #[test]
    fn test_tick_teach_at_very_high_trust() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.5, 0.0, 0.9, true, false);
        assert!(reflexes.iter().any(|r| r.instinct == InstinctType::Teach));
    }

    #[test]
    fn test_tick_mourn_on_peer_death() {
        let mut engine = InstinctEngine::new(default_thresholds());
        engine.tick(0.5, 0.0, 0.5, true, false); // peer alive
        let reflexes = engine.tick(0.5, 0.0, 0.5, false, false); // peer dies
        assert!(reflexes.iter().any(|r| r.instinct == InstinctType::Mourn));
    }

    #[test]
    fn test_tick_no_mourn_if_peer_still_dead() {
        let mut engine = InstinctEngine::new(default_thresholds());
        engine.tick(0.5, 0.0, 0.5, true, false);
        engine.tick(0.5, 0.0, 0.5, false, false); // peer dies
        let reflexes = engine.tick(0.5, 0.0, 0.5, false, false); // still dead
        assert!(!reflexes.iter().any(|r| r.instinct == InstinctType::Mourn));
    }

    #[test]
    fn test_highest_priority_returns_most_urgent() {
        let mut engine = InstinctEngine::new(default_thresholds());
        engine.tick(0.1, 0.0, 0.5, true, false); // survive
        assert!(engine.highest_priority().unwrap().instinct == InstinctType::Survive);
    }

    #[test]
    fn test_highest_priority_none_when_empty() {
        let mut engine = InstinctEngine::new(default_thresholds());
        assert!(engine.highest_priority().is_none());
    }

    #[test]
    fn test_suppress_disables_reflex() {
        let mut engine = InstinctEngine::new(default_thresholds());
        engine.tick(0.1, 0.0, 0.5, true, false); // fires survive
        engine.suppress(InstinctType::Survive);
        assert!(!engine.last_reflexes.iter().any(|r| r.instinct == InstinctType::Survive && r.is_active()));
    }

    #[test]
    fn test_reflex_energy_costs() {
        let survive = Reflex::new(InstinctType::Survive, 1.0);
        let curious = Reflex::new(InstinctType::Curious, 1.0);
        assert!(survive.energy_cost > curious.energy_cost);
    }

    #[test]
    fn test_reflex_urgency_clamped() {
        let r = Reflex::new(InstinctType::Guard, 1.5);
        assert!(r.urgency <= 1.0);
        let r2 = Reflex::new(InstinctType::Guard, -0.5);
        assert!(r2.urgency >= 0.0);
    }

    #[test]
    fn test_reflexes_sorted_by_urgency() {
        let mut engine = InstinctEngine::new(default_thresholds());
        let reflexes = engine.tick(0.1, 0.9, 0.9, false, false);
        for i in 0..reflexes.len().saturating_sub(1) {
            assert!(reflexes[i].urgency >= reflexes[i+1].urgency);
        }
    }

    #[test]
    fn test_history_records_and_retrieves() {
        let mut history = InstinctHistory::new();
        history.record(HistoryEntry { cycle: 1, instinct: InstinctType::Guard, urgency: 0.5, acted: true });
        history.record(HistoryEntry { cycle: 2, instinct: InstinctType::Curious, urgency: 0.3, acted: false });
        let last = history.last_n(1);
        assert_eq!(last.len(), 1);
        assert_eq!(last[0].instinct, InstinctType::Curious);
    }

    #[test]
    fn test_history_frequency() {
        let mut history = InstinctHistory::new();
        for _ in 0..5 {
            history.record(HistoryEntry { cycle: 1, instinct: InstinctType::Guard, urgency: 0.5, acted: true });
        }
        for _ in 0..3 {
            history.record(HistoryEntry { cycle: 2, instinct: InstinctType::Curious, urgency: 0.3, acted: false });
        }
        assert_eq!(history.frequency(InstinctType::Guard), 5);
        assert_eq!(history.frequency(InstinctType::Curious), 3);
    }

    #[test]
    fn test_history_dominant() {
        let mut history = InstinctHistory::new();
        for _ in 0..10 {
            history.record(HistoryEntry { cycle: 1, instinct: InstinctType::Guard, urgency: 0.5, acted: true });
        }
        for _ in 0..3 {
            history.record(HistoryEntry { cycle: 2, instinct: InstinctType::Flee, urgency: 0.8, acted: false });
        }
        assert_eq!(history.dominant(), Some(InstinctType::Guard));
    }

    #[test]
    fn test_history_dominant_empty() {
        let history = InstinctHistory::new();
        assert!(history.dominant().is_none());
    }

    #[test]
    fn test_instinct_type_names() {
        assert_eq!(InstinctType::Survive.name(), "survive");
        assert_eq!(InstinctType::Mourn.name(), "mourn");
        assert_eq!(InstinctType::None.name(), "none");
    }

    #[test]
    fn test_cycle_count_increments() {
        let mut engine = InstinctEngine::new(default_thresholds());
        assert_eq!(engine.cycle_count(), 0);
        engine.tick(0.5, 0.0, 0.5, true, false);
        assert_eq!(engine.cycle_count(), 1);
        engine.tick(0.5, 0.0, 0.5, true, false);
        assert_eq!(engine.cycle_count(), 2);
    }
}

