# ESP32-C3 DuduClock（嘟嘟天气时钟）开发与文档工程

本项目为 ESP32-C3 + 2.4寸 ST7789 LCD 屏幕（DuduClock / 嘟嘟天气时钟）的开发工程集合目录。包含完整的 C/C++ 源码、Rust 裸机工程、官方出厂固件备份及完整的硬件引脚映射文档。

> ⚠️ **强烈推荐阅读**：[DEBUG_LOG_AND_PITFALLS.md](DEBUG_LOG_AND_PITFALLS.md)（记录了此前开发踩过的所有大坑、白屏根因与错误日志，移交 AI 开发必看）。

> **2026-08-29 当前状态**：设备正在运行已实机验证正常的最小 Rust 彩色显示固件。持续灰白的根因已经确认是本地配置误把 TFT SCLK 写成了 GPIO10；这批硬件必须使用 GPIO2。

---

## 目录结构

```text
esp32-2.4Inch/
├── README.md                     # 本开发文档与硬件引脚说明
├── DEBUG_LOG_AND_PITFALLS.md     # 踩坑全记录与调试心得（移交必看）
├── backup/
│   └── DuduClock_2.1.2.bin   # 原厂官方出厂全量固件备份（2.9MB）
├── DuduClock_Firmware/       # 原作者 Arduino C/C++ 完整源码工程
└── esp32c3-display-rust/     # Rust 裸机（no_std + esp-hal）开发工程
```

---

## 硬件规格与引脚映射表

### 1. 核心控制器
* **主控芯片**: ESP32-C3 (RISC-V 32-bit 单核, 160MHz, 4MB Flash)
* **通信与烧录**: 原生 USB-Serial/JTAG（当前 macOS 端口: `/dev/cu.usbmodem1101`）

### 2. 屏幕参数（2.4 寸 TFT LCD）
* **驱动 IC**: ST7789 / ST7789V
* **分辨率**: 240 x 320
* **色彩格式**: RGB565（BGR 颜色顺序，`TFT_BGR`）
* **反相设置**: 关闭反相（`TFT_INVERSION_OFF` / `0x20`）
* **SPI 通信参数**: 27MHz，SPI Mode 0

### 3. 引脚分配（Pinout）

| 信号名称 | GPIO 引脚 | 功能描述 |
| :--- | :--- | :--- |
| **SCLK** | `GPIO 2` | 屏幕 SPI 时钟线（由可正常显示的官方 2.1.2 app 机器码验证） |
| **MOSI** | `GPIO 3`  | 屏幕 SPI 数据输入线 |
| **CS**   | `GPIO 7`  | 屏幕 SPI 片选引脚 |
| **DC**   | `GPIO 4`  | 屏幕数据 / 命令选择引脚 |
| **RST**  | `GPIO 5`  | 屏幕硬件复位引脚 |
| **BL**   | `GPIO 6`  | 背光 NMOS 栅极；该硬件版本保持复位态更稳定 |
| **KEY**  | `GPIO 8`  | 板载功能按键 |
| **SDA**  | `GPIO 8`  | I2C 数据（温湿度传感器 AHT20/SHT30） |
| **SCL**  | `GPIO 9`  | I2C 时钟（温湿度传感器 AHT20/SHT30） |

---

## 常用操作与一键命令

### 1. 一键恢复官方原厂出厂固件
如果开发过程中屏幕显示异常或需要恢复原厂时钟界面，直接在终端执行：
```bash
uv run --with esptool esptool --port /dev/cu.usbmodem1101 write-flash 0x0 backup/DuduClock_2.1.2.bin
```

### 2. Arduino C/C++ 源码开发
项目源码位于 `DuduClock_Firmware/`。
* 依赖库（已配置在本地环境）：`TFT_eSPI`、`OneButton`、`ArduinoJson`、`ArduinoZlib`、`DuduUtil`、`TaskScheduler`。

### 3. Rust 裸机开发
项目源码位于 `esp32c3-display-rust/`。
* 架构：`riscv32imc-unknown-none-elf`
* 底层 HAL：`esp-hal 1.1.2` (`no_std`)
* 图形引擎：`embedded-graphics 0.8`
* 已验证参数：CPU 80MHz；SPI 27MHz / Mode 0；SCLK=2、MOSI=3、CS=7、DC=4、RST=5；GPIO6 保持复位态。
* 实机状态、画面内容、编译和烧录命令见 [`esp32c3-display-rust/README.md`](esp32c3-display-rust/README.md)。

---

## 相关开源资源
* **原作者 GitHub 仓库**: [https://github.com/leezicai/DuduClock_Firmware](https://github.com/leezicai/DuduClock_Firmware)
* **原作者 GitHub Releases**: [https://github.com/leezicai/DuduClock_Firmware/releases/tag/v2.1.2](https://github.com/leezicai/DuduClock_Firmware/releases/tag/v2.1.2)
* **B 站原作者教学视频**: 搜索 `leezicai` 或 `嘟嘟时钟`
* **立创开源硬件平台**: 搜索 `DuduClock`
