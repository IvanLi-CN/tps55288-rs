## Default goal
.DEFAULT_GOAL := help

SHELL := /bin/bash

ROOT_DIR := $(abspath $(CURDIR))
EX_DIR := examples
SELECT_PROBE_SCRIPT := $(ROOT_DIR)/scripts/select-probe.sh

EXAMPLE_STM32G031 := stm32g031g8u6

.PHONY: help select-probe \
        run-stm32g031-fixed run-stm32g031-step run-stm32g031-ext-fb \
        build-stm32g031-fixed build-stm32g031-step build-stm32g031-ext-fb \
        attach-stm32g031-fixed attach-stm32g031-step attach-stm32g031-ext-fb \
        reset-stm32g031

help:
	@echo 'Usage:'
	@echo '  1. eval "$$(make -s select-probe)"'
	@echo '  2. make <target>'
	@echo
	@echo 'STM32G031G8U6 TPS55288 example targets (delegated to examples/$(EXAMPLE_STM32G031)/Makefile):'
	@echo '  build-stm32g031-fixed      - Build fixed_5v (internal FB)'
	@echo '  build-stm32g031-step       - Build step_vout (internal FB)'
	@echo '  build-stm32g031-ext-fb     - Build ext_fb_sw2303 (external FB + SW2303)'
	@echo '  run-stm32g031-fixed        - Flash + run fixed_5v'
	@echo '  run-stm32g031-step         - Flash + run step_vout'
	@echo '  run-stm32g031-ext-fb       - Flash + run ext_fb_sw2303'
	@echo '  attach-stm32g031-fixed     - Attach to fixed_5v'
	@echo '  attach-stm32g031-step      - Attach to step_vout'
	@echo '  attach-stm32g031-ext-fb    - Attach to ext_fb_sw2303'
	@echo '  reset-stm32g031            - Reset target MCU'
	@echo
	@echo 'Examples:'
	@echo '  make run-stm32g031-ext-fb'
	@echo '  make build-stm32g031-fixed'

select-probe:
	@$(SELECT_PROBE_SCRIPT)

build-stm32g031-fixed:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) build-fixed

build-stm32g031-step:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) build-step

build-stm32g031-ext-fb:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) build-ext-fb

run-stm32g031-fixed:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) run-fixed

run-stm32g031-step:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) run-step

run-stm32g031-ext-fb:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) run-ext-fb

attach-stm32g031-fixed:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) attach-fixed

attach-stm32g031-step:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) attach-step

attach-stm32g031-ext-fb:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) attach-ext-fb

reset-stm32g031:
	$(MAKE) -C $(EX_DIR)/$(EXAMPLE_STM32G031) reset
