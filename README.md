# Hymo - Next Generation Android Hybrid Mount Engine

![Language](https://img.shields.io/badge/Language-C++-00599C?style=flat-square&logo=cplusplus)
![Platform](https://img.shields.io/badge/Platform-Android%20(KernelSU)-3DDC84?style=flat-square&logo=android)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

> **Hymo** is an experimental hybrid mount meta-module designed for KernelSU. It refactors core logic with native C++ and introduces HymoFS, which attempts to hijack the kernel file system to use VFS mapping. Note that this approach is intrusive and may cause system instability.

---
**[ 🇨🇳 中文 ](docs/README_ZH.md)**

## Core Architecture

Hymo is a native daemon designed to handle complex mounting scenarios, though it may introduce new compatibility issues.

### 1. Native C++ Engine
*   **Native Implementation**: Core logic is written in C++ to avoid Shell script dependencies, though this introduces binary compatibility risks.
*   **Direct System Calls**: Uses Linux Mount APIs like `fsopen`, `fsconfig`, `fsmount`, `move_mount` directly. This bypasses standard `mount` safeguards and may lead to undefined behavior if misused.
*   **Startup Impact**: While optimized, the module loading process inevitably adds overhead to the system boot time and may delay startup.

### 2. HymoFS & Multi-Mode Engine
Hymo introduces experimental **HymoFS** technology, building a complex mount strategy:
*   **HymoFS (Kernel Mode)**: Requires patching the kernel source to open a kernel interface. It hijacks the underlying file system to implement file mapping. This is a significant kernel modification and may compromise system integrity or cause data loss.
*   **OverlayFS (General Mode)**: The standard solution, utilizing kernel OverlayFS features for file-level merging.
*   **Magic Mount (Compatibility Mode)**: A traditional Bind Mount fallback for old kernels or special partitions.
*   **Dynamic Decision**: The daemon attempts to detect the environment to select a mode, or users can force a specific mode via WebUI.

### 3. Synchronization
*   **Incremental Update**: Attempts to compare module `module.prop` and file timestamps at startup.
*   **I/O Operations**: Synchronizes changed module files to the working directory. While intended to reduce I/O, the checking process itself consumes resources and may wear flash memory over time.

### 4. Storage Backend
*   **Space Management**: Detects kernel Tmpfs support. If supported, it uses the in-memory file system, which consumes system RAM and may lead to out-of-memory (OOM) situations on devices with limited memory.
*   **Tmpfs Usage**: Builds module images in memory at runtime. This is volatile and data will be lost on reboot.
*   **Ext4 Image Fallback**: Falls back to `modules.img` loopback image if Tmpfs is unavailable, which consumes permanent storage space.

---

## Build & Install

Hymo uses a standard Makefile build system and supports cross-compilation.

**Prerequisites**:
*   Android NDK (r25+)
*   Node.js & npm (for building WebUI)
*   Make & Zip

**HymoFS Patch**

HymoFS provides a smart `setup.sh` script for easy integration into your kernel source.

### One-Click Install
```bash
curl -LSs https://raw.githubusercontent.com/Anatdx/HymoFS/main/setup.sh | bash -s defconfig arch/arm64/configs/gki_defconfig
```

### Features
*   **Branch Selection**: Attempts to detect your kernel version (6.1 or 6.6) and switch branches. This detection may fail on non-standard kernels.
*   **KernelSU Detection**: Tries to detect KernelSU.
*   **SUSFS Integration**: Attempts to apply compatibility patches if SUSFS is detected. This integration is fragile and may break with updates to either project.

**Build Commands**:
```bash
# Compile all architectures and package
make zip

# Compile arm64 only and generate test package
make testzip
```

**Install**:
The generated zip package can be flashed in KernelSU Manager, assuming the manager supports the format.

---

## CLI Usage (hymod)

Hymo includes a command-line tool `hymod` for debugging the daemon and HymoFS rules.

### Basic Usage
```bash
hymod [OPTIONS] [COMMAND]
```

### Commands
*   `mount`: Attempt to mount all modules (Default action).
*   `modules`: List modules that Hymo believes are active.
*   `storage`: Show current storage status.
*   `reload`: Attempt to reload HymoFS mappings.
*   `clear`: Clear all HymoFS mappings (Use this if the system becomes unstable).
*   `list`: List active HymoFS kernel rules.
*   `version`: Show protocol version.
*   `gen-config`: Generate a default configuration file.
*   `show-config`: Display the current configuration.
*   `add <mod_id>`: Manually add a module's rules.
*   `delete <mod_id>`: Manually remove a module's rules.
*   `set-mirror <path>`: Set custom mirror path.
*   `raw <cmd> ...`: Execute raw HymoFS low-level commands. **Warning: Improper use will crash the system.**

### Options
*   `-c, --config FILE`: Specify a custom config file path.
*   `-m, --moduledir DIR`: Specify the module directory.
*   `-v, --verbose`: Enable verbose logging.
*   `-p, --partition NAME`: Add a partition to scan.

---

## Credits

*   **KernelSU**: Root solution.
*   **Libcxx**: Android NDK C++ standard library.
*   **Contributors**: Thanks for testing.

---

> **Disclaimer**: This project involves dangerous low-level system modifications. **Data loss is likely.** Please ensure data backup before use. The developer is not responsible for any damage, bricked devices, or nuclear war caused by using this module.
