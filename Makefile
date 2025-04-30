# ===========================
#        Proton Makefile
# ===========================

# Project settings
CARGO=cargo
BUILD_MODE ?= debug             # can be overridden with: make BUILD_MODE=release
TARGET_DIR=target/$(BUILD_MODE)

# Binary names
ASSEMBLER_BIN=assembler
MACHINE_BIN=machine

# Arguments (can be overridden)
ASM_INPUT ?=examples/test.qasm
ASM_OUTPUT ?=build/output.out
QASM_FILE ?=build/output.out

# Colors for output
GREEN=\033[0;32m
RED=\033[0;31m
NC=\033[0m

# ========== Targets ==========

.PHONY: all build clean run-assembler run-machine

all: build

build:
	@echo -e "$(GREEN)[BUILD] Compiling $(ASSEMBLER_BIN) and $(MACHINE_BIN) in $(BUILD_MODE) mode...$(NC)"
	@$(CARGO) build --bin $(ASSEMBLER_BIN) --release
	@$(CARGO) build --bin $(MACHINE_BIN) --release
	@echo -e "$(GREEN)[DONE] Build completed.$(NC)"

run-assembler: build
	@echo -e "$(GREEN)[RUN] Running $(ASSEMBLER_BIN) with: $(ASM_INPUT) -> $(ASM_OUTPUT)$(NC)"
	@$(CARGO) run --bin $(ASSEMBLER_BIN) -- $(ASM_INPUT) $(ASM_OUTPUT) || \
		(echo -e "$(RED)[ERROR] Assembler failed.$(NC)" && exit 1)

run-machine: build
	@echo -e "$(GREEN)[RUN] Running $(MACHINE_BIN) with: $(QASM_FILE)$(NC)"
	@$(CARGO) run --bin $(MACHINE_BIN) -- $(QASM_FILE) || \
		(echo -e "$(RED)[ERROR] Machine execution failed.$(NC)" && exit 1)

clean:
	@echo -e "$(GREEN)[CLEAN] Removing target directory...$(NC)"
	@$(CARGO) clean
	@echo -e "$(GREEN)[DONE] Cleaned up.$(NC)"
