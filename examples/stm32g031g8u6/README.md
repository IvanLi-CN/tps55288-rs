# STM32G031G8U6 Example (Planned)

## Goal
Demonstrate TPS55288 control on STM32G031G8U6: PPS-like dynamic VOUT steps (5/9/12/15/20V), ILIM config, fault polling.

## Hardware Assumptions
- STM32G031G8U6 board with I2C1 available; SCL/SDA to TPS55288.
- EN pin tied to MCU GPIO (optional). IRQ/INT (FB/INT pin) to GPIO input if routed.
- VIN bench supply (9–20 V). VOUT to load.
- I2C pull-ups present.

## Software Stack (to be written)
- Crate deps: `embedded-hal` / `embedded-hal-async` (choose sync vs async build), MCU HAL (e.g., stm32g0xx-hal or embassy-stm32).
- Use `tps55288-rs` driver: init -> set VOUT/ILIM -> loop status check -> step VOUT.

## Planned Flow
1. Init clocks, I2C, optional GPIO for EN.
2. Create driver with default address 0x74, call `init()` (or `init_async`).
3. Set VOUT to 5 V, ILIM to board-safe value (e.g., 3–4 A), select PFM or PWM as desired.
4. Loop stepping VOUT through 9/12/15/20 V with delays; read status/fault; log via UART/RTT.
5. Optional: respond to button/serial to change VOUT/ILIM dynamically.

## Build/Flash (to be added)
- Add Cargo configuration with target/runner (e.g., `thumbv6m-none-eabi`).
- Provide Makefile or justfile for build/flash using openocd/probe-rs.

