# Hymo - 下一代 Android 混合挂载引擎

![Language](https://img.shields.io/badge/Language-C++-00599C?style=flat-square&logo=cplusplus)
![Platform](https://img.shields.io/badge/Platform-Android%20(KernelSU)-3DDC84?style=flat-square&logo=android)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

> **Hymo** 是一个专为 KernelSU 设计的实验性混合挂载元模块。它采用原生 C++ 重构核心逻辑，引入劫持内核文件系统的 HymoFS。请注意，这种通过 VFS 映射而非标准挂载的方式属于侵入性修改，可能导致系统不稳定。

---
**[ 🇺🇸/🇬🇧 English ](../README.md)**

## 核心架构 (Core Architecture)

Hymo 是一个原生守护进程（Daemon），旨在处理复杂的挂载场景，尽管这可能会引入新的兼容性问题。

### 1. 原生 C++ 引擎
*   **原生实现**：核心逻辑由 C++ 编写以减少 Shell 依赖，但这可能引入二进制兼容性问题。
*   **直接系统调用**：直接使用 `fsopen`, `fsconfig`, `fsmount`, `move_mount` 等 Linux Mount API。这绕过了标准 `mount` 命令的安全检查，若使用不当可能导致未定义行为。
*   **启动影响**：尽管经过优化，模块加载过程仍不可避免地会增加系统启动开销，并可能导致启动延迟。

### 2. HymoFS 与多模式引擎 (HymoFS & Multi-Mode Engine)
Hymo 引入了实验性的 **HymoFS** 技术，构建了复杂的挂载策略：
*   **HymoFS (内核模式)**：需要对内核源码打补丁以开放内核接口。它直接劫持底层文件系统以实现文件映射。这是一种重大的内核修改，可能会破坏系统完整性或导致数据丢失。
*   **OverlayFS (通用模式)**：标准环境下的常规方案，利用内核 OverlayFS 特性实现文件级合并。
*   **Magic Mount (兼容模式)**：针对旧内核或特殊分区的传统 Bind Mount 兜底方案。
*   **动态决策**：守护进程尝试检测环境以选择模式，用户也可通过 WebUI 强制指定。

### 3. 同步机制 (Synchronization)
*   **增量更新**：尝试在启动时比对模块的 `module.prop` 和文件时间戳。
*   **I/O 操作**：将变更的模块文件同步到工作目录。虽然旨在减少 I/O，但检查过程本身会消耗资源，且频繁操作仍可能加速闪存磨损。

### 4. 存储后端 (Storage Backend)
*   **空间管理**：检测内核 Tmpfs 支持情况。如果支持，则使用内存文件系统，这会占用系统 RAM，在小内存设备上可能导致内存不足 (OOM)。
*   **Tmpfs 使用**：运行时在内存中构建模块镜像。这是易失性的，重启后数据即丢失。
*   **Ext4 镜像回退**：若不支持 Tmpfs，则回退到 `modules.img` 环回镜像，这将占用永久存储空间。

---


## 编译与安装 (Build & Install)

Hymo 使用标准的 Makefile 构建系统，支持交叉编译。

**前置要求**：
*   Android NDK (r25+)
*   Node.js & npm (用于构建 WebUI)
*   Make & Zip

**HymoFS补丁**

HymoFS 提供了一个智能的 `setup.sh` 脚本，用于轻松集成到您的内核源码中。

### 一键安装
```bash
curl -LSs https://raw.githubusercontent.com/Anatdx/HymoFS/main/setup.sh | bash -s defconfig arch/arm64/configs/gki_defconfig
```

### 功能特性
*   **分支选择**: 尝试检测您的内核版本（6.1 或 6.6）并切换分支。此检测在非标准内核上可能会失败。
*   **KernelSU 检测**: 尝试检测 KernelSU。
*   **SUSFS 集成**: 如果检测到 SUSFS，则尝试应用兼容补丁。此集成非常脆弱，任一项目的更新都可能导致其失效。

**编译命令**：
```bash
# 编译所有架构并打包
make zip

# 仅编译 arm64 并生成测试包
make testzip
```

**安装**：
生成的 zip 包可以在 KernelSU Manager 中刷入（假设管理器支持该格式）。

---

## 命令行工具 (CLI Usage)

Hymo 包含一个命令行工具 `hymod`，用于调试守护进程和 HymoFS 规则。

### 基本用法
```bash
hymod [选项] [命令]
```

### 命令列表
*   `mount`: 尝试挂载所有模块（默认操作）。
*   `modules`: 列出 Hymo 认为处于活跃状态的模块。
*   `storage`: 显示当前存储状态。
*   `reload`: 尝试重载 HymoFS 映射。
*   `clear`: 清空所有 HymoFS 映射（如果系统变得不稳定，请使用此选项）。
*   `list`: 列出活跃的 HymoFS 内核规则。
*   `version`: 显示协议版本。
*   `gen-config`: 生成默认配置文件。
*   `show-config`: 显示当前配置。
*   `add <mod_id>`: 手动添加模块规则。
*   `delete <mod_id>`: 手动删除模块规则。
*   `set-mirror <path>`: 设置自定义镜像路径。
*   `raw <cmd> ...`: 执行原始 HymoFS 底层命令。**警告：使用不当会导致系统崩溃。**

### 选项
*   `-c, --config FILE`: 指定自定义配置文件路径。
*   `-m, --moduledir DIR`: 指定模块目录。
*   `-v, --verbose`: 启用详细日志。
*   `-p, --partition NAME`: 添加要扫描的分区。

---

## 致谢 (Credits)

*   **[Meta-Hybrid Mount](https://github.com/YuzakiKokuban/meta-hybrid_mount)**: 原型基础。
*   **KernelSU**: Root 解决方案。
*   **Libcxx**: Android NDK C++ 标准库。
*   **贡献者**: 感谢测试。

---

> **免责声明**: 本项目涉及危险的系统底层修改。**数据丢失的可能性很高。** 请务必在使用前备份数据。开发者不对因使用本模块导致的任何损坏、设备变砖或核战争负责。
