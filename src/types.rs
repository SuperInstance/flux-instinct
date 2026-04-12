#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstinctType {
    Survive = 0,
    Flee = 1,
    Guard = 2,
    Report = 3,
    Hoard = 4,
    Cooperate = 5,
    Teach = 6,
    Curious = 7,
    Mourn = 8,
    Evolve = 9,
    None = 99,
}

impl InstinctType {
    pub fn name(self) -> &'static str {
        match self {
            InstinctType::Survive => "survive",
            InstinctType::Flee => "flee",
            InstinctType::Guard => "guard",
            InstinctType::Report => "report",
            InstinctType::Hoard => "hoard",
            InstinctType::Cooperate => "cooperate",
            InstinctType::Teach => "teach",
            InstinctType::Curious => "curious",
            InstinctType::Mourn => "mourn",
            InstinctType::Evolve => "evolve",
            InstinctType::None => "none",
        }
    }
}

impl Default for InstinctType {
    fn default() -> Self {
        InstinctType::None
    }
}
