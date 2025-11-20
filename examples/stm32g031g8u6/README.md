# STM32G031G8U6 Example (Plan)

This example will demonstrate TPS55288 control from an STM32G031G8U6 (I2C) board. Code is not implemented yet; this document captures the intended wiring and flow.

## Hardware Assumptions
- MCU: STM32G031G8U6 (I2C1 on PA9/PA10 or board-specific pins).
- Power: VIN 5–20 V bench supply; TPS55288 VOUT to electronic load.
- Control pins: EN tied to MCU GPIO (optional), IRQ/PG (if routed) to GPIO inputs.
- I2C pull-ups present; shared bus OK.

## Planned Software Stack
- STM32 HAL or Embassy (to be decided based on target board support in this repo).
- `embedded-hal` traits for driver; async path if Embassy is chosen.

## Demo Flow
1. Init I2C + TPS55288 driver, ensure device enabled.
2. Set mode to auto (buck/boost) and configure soft-start/frequency per board limits.
3. Program VOUT steps (e.g., 5 V → 9 V → 12 V → 15 V → 20 V) with settle delays.
4. Set current limit appropriate to board (e.g., 2–4 A) and validate via status bits.
5. Poll status/fault registers; clear faults on detection; log via RTT/UART.
6. Optional: respond to button/serial commands to change VOUT/ILIM dynamically.

## Build/Flash (to be filled)
- Target triple, runner, and memory.x will be documented alongside the example code.
- Makefile/cargo runner instructions will be added when the code is implemented.
