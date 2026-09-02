# DuduClock 最小 Rust 彩色显示固件

## 已验证状态

2026-08-29 已在 ESP32-C3 rev 0.4、4MB Flash、ST7789V 240×320 实机上验证成功。屏幕显示彩色图像并持续刷新，不再灰白。这个工程只负责证明 Rust 可以正常点屏，不包含天气、网络、时钟或配网功能。

当前设备即运行此固件。实机确认来自用户现场观察；开发端没有摄像头或 LCD 帧缓冲回读能力，因此没有远程实机截图。

## 绝对不能改错的参数

- CPU：80MHz
- SPI：27MHz，Mode 0
- SCLK：GPIO2（持续灰白问题的关键修复；GPIO10 是错误配置）
- MOSI：GPIO3
- CS：GPIO7
- DC：GPIO4
- RST：GPIO5
- GPIO6：最小显示固件不操作
- 像素格式：RGB565，MADCTL BGR
- 反相：发送 `INVOFF`（命令 `0x20`）

## 当前画面

- 黑色背景和青色外框
- `Rust on ESP32-C3`
- `DuduClock 2.4-inch ST7789`
- `Arch: RISC-V 32-bit (RV32IMC)`
- `HAL : esp-hal 1.1.2 (no_std)`
- `Engine: embedded-graphics`
- 红、黄、绿、青、蓝、洋红六色条
- 青色圆环与洋红色实心圆
- `AI Agent Online`
- `Heartbeat Loop: Active`
- 每 500ms 变化一次的橙色进度条；串口同时输出 `Tick #N`

源码中没有 `cloud` 或 `action` 字样。小字号在 TN 屏上可能产生近似观感。

## 编译

首次本地编译前，请复制 `src/config.example.rs` 为 `src/config.rs`，再填写自己的 Wi-Fi 配置。`src/config.rs` 已加入 `.gitignore`，不会上传到 GitHub。

本机 Homebrew Rust 与 rustup target 路径不共用。直接运行 `cargo build` 可能出现 `can't find crate for core`，因此使用：

```bash
RUSTC=/Users/yingqing/.rustup/toolchains/1.97.1-aarch64-apple-darwin/bin/rustc \
  rustup run 1.97.1 cargo build --release
```

已验证 release ELF：

```text
target/riscv32imc-unknown-none-elf/release/esp32c3-display-rust
SHA-256 0f598c037a10e36c1a8b1bc9c0240c96e9c615d984e712b130231a518eb35cf9
烧录应用占用 110128 / 4128768 bytes (2.67%)
```

## 烧录与串口

```bash
espflash flash --chip esp32c3 --port /dev/cu.usbmodem11401 \
  target/riscv32imc-unknown-none-elf/release/esp32c3-display-rust

espflash monitor --chip esp32c3 --port /dev/cu.usbmodem11401
```

如果烧录后显示状态可疑，完整拔掉 USB，等待至少 10 秒，再重新插入确认。恢复官方完整固件的命令见上级目录 `README.md`。

## 给后续模型的最短结论

不要重新猜引脚。此前灰白不是加密、官方私有库或 Rust 无法驱屏，而是 TFT SPI 时钟配置错误：GPIO10 错，GPIO2 对。当前 `src/main.rs` 是已经实机显示正常的基线；开发新功能时先保持显示初始化和引脚不变。
