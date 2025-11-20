# TPS55288 Buck-Boost Driver (Rust)

Rust driver skeleton for TI TPS55288 36 V / 16 A four-switch buck-boost converter with I2C control. This repo follows the patterns proven in `sc8815-rs` (no-std, sync/async via `maybe-async-cfg`, type-safe registers, strong error handling). Implementation is not yet written—this repo captures the design, TODOs, and example layout to accelerate bring-up.

## Status
- Planning and structure only; no driver code yet.
- Datasheet-driven register modeling, API surface, and example wiring defined in `DESIGN.md`.
- STM32G031G8U6 reference example reserved under `examples/stm32g031g8u6/`.

## Scope & Goals
- Provide a safe, no-std Rust driver with optional async + defmt.
- Cover I2C register map: output voltage/current limits, mode control (buck/boost/auto), PPS-style programmable voltage steps, protections, status/interrupts.
- Ship integration example for STM32G031G8U6 demonstrating PPS-like VOUT control and fault polling.

## Repo Layout (planned)
- `src/` — modules for registers, data types, driver core, and errors (placeholders now).
- `examples/stm32g031g8u6/` — board wiring notes and build plan for the reference example.
- `DESIGN.md` — detailed architecture and API plan.
- `TODO.md` — execution checklist.

## Next Steps
See `TODO.md` for the ordered task list. Implementation begins with datasheet extraction and register modeling, followed by driver scaffolding and the STM32 example.
