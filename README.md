# Kryptex

A modular crypto perpetuals signal generation and execution engine built in Rust.

## Overview

Kryptex is designed to:
1. Receive market data from exchanges (initially Hyperliquid)
2. Calculate technical indicators
3. Generate trading signals with recommended stop loss (SL) and take profit (TP) percentages
4. Execute Long/Short orders in perpetual futures
5. Maintain modularity to allow changing exchanges without altering core logic

## Current Status: Phase 2 Scaffold

The POC has been archived in favor of the production-ready architecture defined in the [@Kryptex RFC](https://github.com/lucastosetto/kryptex/wiki/1.-RFC-%E2%80%90-Kryptex:-Crypto-Perps-Signal-&-Execution-Engine):

- ✅ Removed CLI runner, prototype weighting logic, and temporary helpers
- ✅ Created the long-term module tree (core, indicators, signals, models, services, strategies, evaluation, common)
- ✅ Preserved configuration and persistence crates for reuse
- 🔄 Implementing native indicator and signal engines within the new layers
- 🔜 Wiring exchange adapters, execution engine, and orchestration services

## Architecture

```
┌─────────────────┐
│ Hyperliquid WS  │─────┐
└───────────┬─────┘     │ Future adapters
            │            │
            ▼            │
    ┌───────────────┐
    │ Market Data    │
    │   Pipeline     │
    └───────┬────────┘
            │ Candles / Indicators (POC)
            ▼
   ┌─────────────────┐
   │ Indicator Engine │
   └────────┬────────┘
            │ Signals
            ▼
  ┌────────────────────────┐
  │ Signal Interpreter      │
  │ + SL/TP Recommendations │
  └──────────┬─────────────┘
             ▼
      (Future) Trade Executor
             ▼
          Unified DB
```

## Project Structure

```
src/
  common/               # Shared helpers (math, time, serialization)
  config/               # Configuration management (unchanged from POC)
  core/                 # Bootstrap/orchestration entry points
  db/                   # Persistence adapters (SQLite)
  evaluation/           # Signal scoring and validation utilities
  indicators/           # Native indicator implementations (MACD, RSI, etc.)
  models/               # Shared DTOs for indicators, signals, execution
  services/             # Long-lived services (market data, persistence facades)
  signals/              # Signal evaluation engine coordination
  strategies/           # Strategy definitions that use indicators + services
  lib.rs                # Crate root exposing layered modules
```

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Build

```bash
cargo check
```

> The repository currently exposes library modules only; no CLI binary ships with the Phase 2 scaffold.

## Working with the Scaffold

The following layers now exist as empty modules that will be filled in during subsequent issues:

- **`core/`** – bootstrap/orchestration glue.
- **`indicators/`** – native indicator engines and candle aggregation.
- **`signals/`** – interprets indicator outputs into actionable signals.
- **`models/`** – shared DTOs across indicators, signals, execution, and persistence.
- **`services/`** – data feeds, persistence coordinators, and adapters.
- **`strategies/`** – named strategies composed of indicators + services.
- **`evaluation/`** – scoring, simulation harnesses, and quality gates.
- **`common/`** – cross-cutting utilities (math/time/helpers).

## Validation

Phase 1 scenario tests and CLI spot checks have been removed along with the POC. Use `cargo check` during development until new automated suites land with the production modules.

### Persistence

Signals are automatically stored in `kryptex_signals.db`:

```rust
use kryptex::db::SignalDatabase;

let db = SignalDatabase::new("kryptex_signals.db")?;
db.store_signal(&signal)?;

let all_signals = db.get_all_signals()?;
let btc_signals = db.get_signals_by_symbol("BTC")?;
```

## Configuration

Default configuration values:

- Default SL: 2%
- Default TP: 4%
- RSI Overbought: 70
- RSI Oversold: 30
- Min Confidence: 0.5

Customize via `Config::new()` or load from JSON file.

## Implementation Roadmap

### ✅ Phase 1 — POC (Current)
- Receive external indicators
- Generate LONG/SHORT signal + SL/TP + reasons
- SQLite persistence

### 🔜 Phase 2 — Native Indicator Module
- Implement MACD, RSI, ATR, volatility
- Candle aggregation: 1m, 5m, 15m, 1h
- Divergences and candle patterns

### 🔜 Phase 3 — Hyperliquid Adapter
- WebSocket market data
- Funding rate fetching
- OHLC reconstruction
- Ed25519 authentication
- Order submission preparation

### 🔜 Phase 4 — Execution Engine
- Order builder
- Trade manager
- Risk manager
- Automatic SL/TP placement
- Trade state machine

### 🔜 Phase 5 — Optional Future Exchanges
- Adapter structure allows easy integration

### 🔜 Phase 6 — Dashboard & Backtester
- Web dashboard (Leptos/Tauri)
- Backtesting engine with historical candles
- Signal performance visualization

## Dependencies

- `serde` / `serde_json` - Serialization
- `rusqlite` - SQLite database
- `chrono` - Timestamps

## Design Principles

- **Modularity**: Exchange adapters can be swapped without changing core logic
- **Precision**: Uses `f64` for all numeric values
- **Extensibility**: Clear separation between signal generation and execution
- **Self-documenting**: Minimal comments, code should be clear

## License

This project is released under the MIT License. See [LICENSE.md](LICENSE.md)
for the full text and terms.

## Contributing

Contributions are welcome! Please read
[CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and quality checklist
before opening a pull request.

