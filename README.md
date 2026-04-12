# flux-instinct

Hardwired reflex system for the FLUX VM, matching instinct-c.

A Rust implementation of a priority-based instinct engine that evaluates biological-style reflexes (survive, flee, guard, hoard, cooperate, teach, curious, mourn, evolve) based on energy, threat, trust, and idle-cycle inputs.

## Quick Start

```rust
use flux_instinct::*;

let mut engine = InstinctEngine::new(Thresholds::default());
let reflexes = engine.tick(energy_frac: 0.5, threat: 0.3, trust: 0.7, peer_alive: true, has_work: false);

if let Some(r) = engine.highest_priority() {
    println!("Acting on: {} (urgency: {})", r.instinct.name(), r.urgency);
}
```

## Instincts (priority order)

| # | Instinct | Trigger |
|---|----------|---------|
| 0 | Survive | Energy ≤ critical (0.15) |
| 1 | Flee | Threat > high (0.7) |
| 2 | Guard | Has work + energy OK |
| 3 | Report | Anomaly detected (elevated threat) |
| 4 | Hoard | Energy ≤ below (0.4) but not critical |
| 5 | Cooperate | Trust > 0.6 |
| 6 | Teach | Trust > 0.8 |
| 7 | Curious | Every 100 idle cycles |
| 8 | Mourn | Peer just died |
| 9 | Evolve | Every 500 idle cycles |
