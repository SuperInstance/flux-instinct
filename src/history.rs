use crate::types::InstinctType;

const MAX_ENTRIES: usize = 256;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub cycle: u64,
    pub instinct: InstinctType,
    pub urgency: f32,
    pub acted: bool,
}

pub struct InstinctHistory {
    entries: Vec<HistoryEntry>,
    head: usize,
    count: usize,
}

impl InstinctHistory {
    pub fn new() -> Self {
        Self {
            entries: vec![HistoryEntry { cycle: 0, instinct: InstinctType::None, urgency: 0.0, acted: false }; MAX_ENTRIES],
            head: 0,
            count: 0,
        }
    }

    pub fn record(&mut self, entry: HistoryEntry) {
        self.entries[self.head] = entry;
        self.head = (self.head + 1) % MAX_ENTRIES;
        if self.count < MAX_ENTRIES {
            self.count += 1;
        }
    }

    pub fn last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        let n = n.min(self.count);
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let idx = (self.head + MAX_ENTRIES - 1 - i) % MAX_ENTRIES;
            result.push(&self.entries[idx]);
        }
        result
    }

    pub fn frequency(&self, t: InstinctType) -> u32 {
        self.entries.iter().take(self.count).filter(|e| e.instinct == t).count() as u32
    }

    pub fn dominant(&self) -> Option<InstinctType> {
        if self.count == 0 {
            return None;
        }
        let mut counts = std::collections::HashMap::new();
        for e in self.entries.iter().take(self.count) {
            if e.instinct != InstinctType::None {
                *counts.entry(e.instinct).or_insert(0u32) += 1;
            }
        }
        counts.into_iter().max_by_key(|(_, c)| *c).map(|(t, _)| t)
    }
}

impl Default for InstinctHistory {
    fn default() -> Self {
        Self::new()
    }
}
