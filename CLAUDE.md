# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
cargo build                                                    # Debug build
cargo test --all                                               # Run all workspace tests
cargo test -p tgvmax-core                                      # Test core library only
cargo test -p tgvmax-cli                                       # Test CLI only
cargo test --all -- test_name                                  # Run a single test by name
cargo fmt --all --check                                        # Check formatting
cargo fmt --all                                                # Apply formatting
cargo clippy --all-targets --all-features -- -D warnings       # Lint (warnings = errors)
cargo run -p tgvmax-cli -- train search -f Paris -t Lyon -d 01/04/2026  # Run CLI
```

**CI requires all three to pass:** `fmt --check`, `clippy -D warnings`, `test --all`.

## Architecture

Rust workspace with two crates:

- **tgvmax-core** — Library: API client, data models, error types. No CLI dependencies.
- **tgvmax-cli** — Binary: CLI interface, station caching, output formatting. Depends on tgvmax-core.

### Key abstraction

`TgvmaxClient` trait (core/client.rs) defines `list_stations()` and `search_trains()`. `OpenDataClient` (core/opendata.rs) implements it against the SNCF Open Data API. This trait enables swapping backends or mocking.

### Data flow

1. CLI parses args (clap derive) → resolves station names via fuzzy substring match (case-insensitive, max 5 matches)
2. For each origin/destination pair, calls `OpenDataClient::search_trains()`
3. Two dedup layers: within single API response by `(train_no, departure_time)`, across queries by `train_number` alone
4. Sort by departure, filter past trains if searching today, render table or JSON

### Error handling

- **tgvmax-core**: Custom `TgvmaxError` enum (Http/Parse/Api) via thiserror, with `Result<T>` alias
- **tgvmax-cli**: `anyhow::Result` for top-level ergonomics; non-critical failures (cache writes, individual query failures) logged to stderr as warnings

### API details

- Train search: SNCF Open Data v1 search endpoint, fields use French names (`origine`, `heure_depart`, `od_happy_card`)
- Station list: v2 group_by aggregation endpoint
- Hardcoded row limits (100 trains, 500 stations) — truncation warnings printed to stderr
- Date format in CLI is DD/MM/YYYY (French convention), converted to YYYY-MM-DD for API

### Caching

Station list cached at `~/.config/tgvmax/stations_cache.json` with 24-hour TTL. Force refresh with `--refresh` flag.
