#[derive(Clone, Debug)]
pub struct Thresholds {
    pub energy_below: f32,
    pub energy_critical: f32,
    pub threat_high: f32,
    pub trust_cooperate: f32,
    pub trust_teach: f32,
    pub idle_trigger: u32,
    pub evolve_idle: u32,
    pub guard_has_work: bool,
    pub report_anomaly: bool,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            energy_below: 0.4,
            energy_critical: 0.15,
            threat_high: 0.7,
            trust_cooperate: 0.6,
            trust_teach: 0.8,
            idle_trigger: 100,
            evolve_idle: 500,
            guard_has_work: true,
            report_anomaly: true,
        }
    }
}
