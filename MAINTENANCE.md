# Maintenance

## Running Tests

```sh
cargo test
```

## Architecture

- `types.rs` — InstinctType enum with priority ordering
- `thresholds.rs` — Configurable trigger thresholds (Default impl)
- `reflex.rs` — Reflex struct with urgency, energy cost, suppression
- `engine.rs` — InstinctEngine: evaluates instincts per tick, returns sorted reflexes
- `history.rs` — Ring-buffer history (max 256 entries) with frequency/dominant queries

## Extending

- Add new instincts to `InstinctType` (pick a priority number)
- Add evaluation logic in `InstinctEngine::tick()`
- Add energy cost in `Reflex::new()` match arm
- Update history tracking if needed

## Compatibility

Matches instinct-c API surface. Reflexes are sorted by urgency (desc) then instinct priority (asc).
