# Configuration variables
BINARY_NAME  := bgselector-gui
BUILD_PATH   := target/release/$(BINARY_NAME)
INSTALL_DIR  := $(HOME)/bin/pickers/bgselector
INSTALL_PATH := $(INSTALL_DIR)/$(BINARY_NAME)

.PHONY: all build install clean help

all: build

build:
	cargo build --release

install: build
	mkdir -p $(INSTALL_DIR)
	cp $(BUILD_PATH) $(INSTALL_PATH)
	@echo "Installed successfully to: $(INSTALL_PATH)"

clean:
	cargo clean

help:
	@echo "Available commands:"
	@echo "  make         - Build in release mode"
	@echo "  make install - Build and install to $(INSTALL_DIR)"
	@echo "  make clean   - Remove build artifacts"
