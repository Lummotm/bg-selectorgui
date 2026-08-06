# Variables de configuración
BINARY_NAME := bgselector-gui
BUILD_PATH := target/release/$(BINARY_NAME)
INSTALL_DIR := $(HOME)/bin/pickers/bgselector

.PHONY: all build optimize install clean help

all: build optimize

build:
	cargo build --release

optimize: build
	@if command -v upx; then \
		echo "Comprimiendo binario con UPX..."; \
		upx --best --ultra-brute $(BUILD_PATH) || true; \
	else \
		echo "UPX no está instalado, omitiendo optimización."; \
	fi
	@echo "Tamaño del binario:"
	@du -h $(BUILD_PATH)

install: all
	mkdir -p $(INSTALL_DIR)
	install -m 755 $(BUILD_PATH) $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Instalado con éxito en: $(INSTALL_DIR)/$(BINARY_NAME)"

clean:
	cargo clean

help:
	@echo "Comandos disponibles:"
	@echo "  make         - Compila en release y comprime con UPX"
	@echo "  make install - Compila, comprime e instala en $(INSTALL_DIR)"
	@echo "  make clean   - Borra artifacts de compilacion"
