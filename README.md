# TPS55288 Buck-Boost Driver (Rust)

 Rust driver skeleton for TI TPS55288 36 V / 16 A four-switch buck-boost converter with I2C control. This repo follows the patterns proven in `sc8815-rs` (no-std, sync/async via `maybe-async-cfg`, type-safe registers, strong error handling). Core driver helpers are in place; example/CI are pending.

## Status
- Sync + async driver helpers implemented (I2C read/write, VOUT/ILIM, VOUT_SR, feedback, cable comp, STATUS decode, init).
- Datasheet-driven register modeling in place; conversion helpers tested.
- STM32G031G8U6 reference example reserved under `examples/stm32g031g8u6/` (code TBD).

## Scope & Goals
- Provide a safe, no-std Rust driver with optional async + defmt.
- Cover I2C register map: output voltage/current limits, mode control (buck/boost/auto), PPS-style programmable voltage steps, protections, status/interrupts.
- Ship integration example for STM32G031G8U6 demonstrating PPS-like VOUT control and fault polling.

## Repo Layout
- `src/` — registers, data types, driver core (sync + async helpers), error handling.
- `examples/stm32g031g8u6/` — board wiring notes and build plan (code pending).
- `docs/tps55288-datasheet.md` — full datasheet extraction with images.
- `DESIGN.md` — architecture and API plan.
- `TODO.md` — execution checklist.

## Next Steps
See `TODO.md` for the ordered task list. Remaining work includes STM32 example, CI/lefthook, defmt display polish, and expanded docs.
