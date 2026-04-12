#[cfg(test)]
mod tests {
    use flux_instinct::*;

    #[test]
    fn instinct_type_name() {
        assert_eq!(InstinctType::Survive.name(), "survive");
        assert_eq!(InstinctType::Flee.name(), "flee");
        assert_eq!(InstinctType::None.name(), "none");
        assert_eq!(InstinctType::Curious.name(), "curious");
    }

    #[test]
    fn instinct_type_ordering() {
        assert!(InstinctType::Survive < InstinctType::Flee);
        assert!(InstinctType::Flee < InstinctType::Guard);
        assert!(InstinctType::Guard < InstinctType::Report);
        assert!(InstinctType::Evolve < InstinctType::None);
    }

    #[test]
    fn thresholds_default_values() {
        let t = Thresholds::default();
        assert!((t.energy_below - 0.4).abs() < f32::EPSILON);
        assert!((t.energy_critical - 0.15).abs() < f32::EPSILON);
        assert!((t.threat_high - 0.7).abs() < f32::EPSILON);
        assert!((t.trust_cooperate - 0.6).abs() < f32::EPSILON);
        assert!((t.trust_teach - 0.8).abs() < f32::EPSILON);
        assert_eq!(t.idle_trigger, 100);
        assert_eq!(t.evolve_idle, 500);
        assert!(t.guard_has_work);
        assert!(t.report_anomaly);
    }

    #[test]
    fn reflex_new_and_active() {
        let r = Reflex::new(InstinctType::Flee, 0.9);
        assert!(r.is_active());
        assert!(!r.suppressed);
        assert!((r.urgency - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn reflex_suppress() {
        let mut r = Reflex::new(InstinctType::Flee, 0.9);
        r.suppress();
        assert!(!r.is_active());
        assert!(r.suppressed);
    }

    #[test]
    fn engine_survive_at_critical_energy() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.1, 0.0, 0.0, true, false);
        assert!(eng.is_firing(InstinctType::Survive));
    }

    #[test]
    fn engine_flee_at_high_threat() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.9, 0.8, 0.0, true, false);
        assert!(eng.is_firing(InstinctType::Flee));
    }

    #[test]
    fn engine_hoard_at_low_but_not_critical() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.3, 0.0, 0.0, true, false);
        assert!(eng.is_firing(InstinctType::Hoard));
        assert!(!eng.is_firing(InstinctType::Survive));
    }

    #[test]
    fn engine_cooperate_with_high_trust() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.9, 0.0, 0.7, true, false);
        assert!(eng.is_firing(InstinctType::Cooperate));
    }

    #[test]
    fn engine_teach_with_very_high_trust() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.9, 0.0, 0.9, true, false);
        assert!(eng.is_firing(InstinctType::Teach));
    }

    #[test]
    fn engine_curious_after_idle_trigger() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        // Run idle_trigger (100) idle cycles
        for _ in 0..100 {
            eng.tick(0.9, 0.0, 0.0, true, false);
        }
        assert!(eng.is_firing(InstinctType::Curious));
    }

    #[test]
    fn engine_evolve_after_evolve_idle() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        for _ in 0..500 {
            eng.tick(0.9, 0.0, 0.0, true, false);
        }
        assert!(eng.is_firing(InstinctType::Evolve));
    }

    #[test]
    fn engine_guard_when_has_work() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        let _refs = eng.tick(0.8, 0.0, 0.0, true, true);
        assert!(eng.is_firing(InstinctType::Guard));
    }

    #[test]
    fn engine_mourn_on_peer_death() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        eng.tick(0.8, 0.0, 0.0, true, false); // peer alive
        let _refs = eng.tick(0.8, 0.0, 0.0, false, false); // peer dies
        assert!(eng.is_firing(InstinctType::Mourn));
    }

    #[test]
    fn engine_highest_priority() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        eng.tick(0.1, 0.0, 0.0, true, false);
        let hp = eng.highest_priority().unwrap();
        assert_eq!(hp.instinct, InstinctType::Survive);
    }

    #[test]
    fn engine_is_firing() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        eng.tick(0.8, 0.0, 0.0, true, false);
        assert!(!eng.is_firing(InstinctType::Flee));
    }

    #[test]
    fn engine_suppress() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        eng.tick(0.1, 0.0, 0.0, true, false);
        assert!(eng.is_firing(InstinctType::Survive));
        eng.suppress(InstinctType::Survive);
        assert!(!eng.is_firing(InstinctType::Survive));
    }

    #[test]
    fn history_record_and_last_n() {
        let mut h = InstinctHistory::new();
        for i in 0..5 {
            h.record(HistoryEntry { cycle: i, instinct: InstinctType::Curious, urgency: 0.3, acted: true });
        }
        let last = h.last_n(3);
        assert_eq!(last.len(), 3);
        assert_eq!(last[0].cycle, 4);
        assert_eq!(last[2].cycle, 2);
    }

    #[test]
    fn history_frequency() {
        let mut h = InstinctHistory::new();
        for _ in 0..3 {
            h.record(HistoryEntry { cycle: 0, instinct: InstinctType::Flee, urgency: 0.5, acted: true });
        }
        for _ in 0..2 {
            h.record(HistoryEntry { cycle: 1, instinct: InstinctType::Guard, urgency: 0.3, acted: false });
        }
        assert_eq!(h.frequency(InstinctType::Flee), 3);
        assert_eq!(h.frequency(InstinctType::Guard), 2);
        assert_eq!(h.frequency(InstinctType::Curious), 0);
    }

    #[test]
    fn history_dominant() {
        let mut h = InstinctHistory::new();
        for _ in 0..5 {
            h.record(HistoryEntry { cycle: 0, instinct: InstinctType::Flee, urgency: 0.5, acted: true });
        }
        for _ in 0..3 {
            h.record(HistoryEntry { cycle: 1, instinct: InstinctType::Guard, urgency: 0.3, acted: false });
        }
        assert_eq!(h.dominant(), Some(InstinctType::Flee));
    }

    #[test]
    fn history_dominant_empty() {
        let h = InstinctHistory::new();
        assert_eq!(h.dominant(), None);
    }

    #[test]
    fn engine_cycle_and_idle_count() {
        let mut eng = InstinctEngine::new(Thresholds::default());
        for _ in 0..10 {
            eng.tick(0.8, 0.0, 0.0, true, false);
        }
        assert_eq!(eng.cycle_count(), 10);
        assert_eq!(eng.idle_count(), 10);
    }
}
