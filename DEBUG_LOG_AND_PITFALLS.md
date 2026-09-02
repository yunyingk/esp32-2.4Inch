# DuduClock ESP32-C3 开发踩坑全记录与调试心得

## 最新已验证状态（后续测试以这里为基线）

### 2026-08-29 18:22:07–18:22:47 CST — 当前设备状态

- 当前设备运行最小 Rust 显示固件 R1，用户已目视确认彩色图像正常，不再灰白。
- 当前画面包含 `Rust on ESP32-C3`、`DuduClock 2.4-inch ST7789`、`Engine: embedded-graphics`、`AI Agent Online`、`Heartbeat Loop: Active`、六色彩条和彩色图形；底部橙色进度条每 500ms 动态变化。
- 用户口述看到类似 “AI engine / cloud / heartbeat loop action” 的内容；与源码小字号文字基本对应。源码中实际没有 `cloud` 或 `action`，准确字符串以上一条为准。
- 本次没有实机照片：开发端只能烧录和读取串口，无法远程读取 LCD 像素或调用摄像头。实机正常结论来自用户现场目视确认。

### 2026-08-29 17:32:52 CST — 官方固件正常基线

- 当前设备运行下载的官方完整固件：`backup/DuduClock_2.1.2.bin`。
- 屏幕状态：彻底拔掉 USB、等待后重新插入，官方界面显示正常。
- 芯片安全状态：`Flags=0`、`Flash_Crypt_Cnt=0`，未启用 Flash Encryption；问题与固件加密或签名无关。
- 回读核验：bootloader、分区表和 `0x10000` 开始的 app 与官方 BIN 逐字节一致。
- 回读的唯一差异位于 `0x9000` NVS 分区，是固件启动后写入配置造成的正常变化。
- 重要结论：烧录后的 USB/RTS 软复位不能作为最终显示结论；ST7789V 可能保留此前的异常状态。此后每次显示测试都必须执行一次完整断电重插。

### 固定测试流程

1. 记录烧录固件、源码版本、编译环境和时间。
2. 烧录完成后先记录串口启动是否正常。
3. 完全拔掉 USB，等待至少 10 秒。
4. 重新插入，记录屏幕颜色、图案、方向和亮度。
5. 测试失败时恢复官方完整 BIN，再按同样的断电流程确认基线。

> 注意：本文下方若有早期推测与本节冲突，以本节的实机验证结果为准。官方 2.1.2 的构建元数据已确认是 Arduino-ESP32 Core 2.0.14；后续“精确复刻”优先使用 2.0.14，而不是早期建议的 2.0.17。

### 2026-08-29 17:34:09–17:34:44 CST — C++ 精确复刻测试 C1

- 基线：B0 官方 2.1.2 正常显示，USB 持续连接。
- 写入范围：仅 app 分区 `0x10000–0x2CDFFF`；官方 bootloader、分区表和 NVS 保留。
- 本地 app：`v2.1.2/DuduClock_Firmware/build-exact-v2.1.2/DuduClock_Firmware.ino.bin`
- app SHA-256：`43910ce23de56be62b5654db6cf9f880609551d1d9946e6ea22c94ab0c5922a4`
- 环境：源码 tag `v2.1.2`，Arduino-ESP32 Core 2.0.14，TFT_eSPI 2.5.43，AirM2M ESP32-C3/Huge APP 参数。
- 烧录结果：写入成功，esptool 数据哈希校验通过；等待完整断电 10 秒后的屏幕观察结果。
- 实机结果：用户完整拔掉 USB、等待后重新插入，屏幕仍为灰白色。
- 结论：C1 失败；软复位/未断电不是本地 C++ 固件灰白的根因。设备暂时保留 C1，等待串口与二进制差异分析。

### 2026-08-29 17:38:24–17:39:00 CST — 官方 app-only 对照 A1

- C1 串口结果：程序正常启动，AP 模式、Wi-Fi 扫描和 Web 服务器均正常，无崩溃。
- 机器码初检：官方与 C1 的 TFT GPIO、SPIClass、TFT 总线及命令写入路径逐条指令一致。
- 写入内容：从官方完整 BIN 的 `0x10000` 原样提取 app，仅覆盖 app 分区，保留与 C1 完全相同的 bootloader、分区表和 NVS。
- 官方 app SHA-256：`8dc3d38747c21f03e9bee8a517943797f588ff435b4e72a065507e085a4db3e9`
- 烧录结果：成功，esptool 数据哈希校验通过；等待完整断电 10 秒后的屏幕结果。
- 实机结果：用户完整断电后重新插入，官方界面正常显示。
- 结论：A1 成功。相同 bootloader、分区表、NVS、供电与操作流程下，仅替换 app 即可恢复显示；故障确定来自本地编译 app 与官方 app 的内容差异。

### 2026-08-29 17:46:33–17:47:02 CST — 最小 C++ 显示测试 C2

- 目的：排除 DuduClock 业务代码、TaskScheduler、网络和大型资源，仅验证本地编译的 Arduino + TFT_eSPI 显示底层。
- 环境：AirM2M ESP32-C3、CPU 80 MHz、Flash 80 MHz、Huge APP、Arduino-ESP32 2.0.14、TFT_eSPI 2.5.43。
- 行为：每 2.5 秒循环红、绿、蓝、白、黑与六色彩条；GPIO6 不操作。
- app：`cpp-tft-espi-diagnostic/build-c2-powercycle/cpp-tft-espi-diagnostic.ino.bin`
- SHA-256：`9264549a9f84dc67e41c51b03b5de93e9691d8d7fbe9284cebb5ca0d856cd93f`
- 烧录结果：成功，esptool 数据哈希校验通过；等待完整断电 10 秒后的观察结果。
- 实机结果：用户完整断电后重新插入并观察，屏幕始终灰白，无彩色循环。
- 结论：C2 失败。可排除 TaskScheduler、网络、天气业务和大型资源；问题收窄到本地 TFT_eSPI/SPI 构建配置或底层 GPIO Matrix/SPI 路由。

### 2026-08-29 — 根因定位：TFT SCLK 引脚错误

- 官方 app 的 `TFT_eSPI::init()` 在调用 `SPIClass::begin(sclk, miso, mosi, ss)` 前加载：`a1=2, a2=3, a3=3, a4=-1`。
- C1/C2 本地 app 对应位置加载：`a1=10, a2=3, a3=3, a4=-1`。
- RISC-V 调用约定中 `a1` 是第一个显式参数 `sclk`，因此官方实际使用 `SCLK=GPIO2`，本地错误配置为 `GPIO10`。
- MOSI=3、CS=7、DC=4、RST=5 由官方机器码确认不变。
- 已将 `DuduClock_Firmware/local_lib/User_Setup.h` 中 `TFT_SCLK` 从 10 修正为 2；等待最小彩条固件重新编译和实机验证。

### 2026-08-29 17:57:12–17:57:27 CST — SCLK 修正验证 C3

- 固件：C2 相同的最小 C++ 彩条程序，仅把 `TFT_SCLK` 从 GPIO10 改为 GPIO2。
- 编译后机器码复核：`SPI.begin()` 参数已变为 `sclk=2, miso=3, mosi=3, ss=-1`，与官方 app 完全一致。
- app：`cpp-tft-espi-diagnostic/build-c3-sclk2/cpp-tft-espi-diagnostic.ino.bin`
- SHA-256：`413a3614bc0c1c06a4dd3f2ad8a69370736377243245f508400e43053b8636bc`
- 烧录结果：成功，esptool 数据哈希校验通过；等待完整断电后的彩条观察结果。
- 实机结果：用户完整断电重插后，屏幕正常循环显示多种纯色，随后显示彩色条纹。
- 结论：C3 成功，根因确认。该硬件的 TFT SPI 时钟为 GPIO2；GPIO10 配置会导致只有背光、画面始终灰白。

### 2026-08-29 18:03:38–18:04:13 CST — 完整 C++ 固件验证 C4

- 源码：官方 tag `v2.1.2`，仅通过 TFT_eSPI 配置将 SCLK 修正为 GPIO2。
- 编译环境：Arduino-ESP32 2.0.14、TFT_eSPI 2.5.43、AirM2M ESP32-C3、CPU/Flash 80 MHz、Huge APP。
- 编译后机器码复核：`SPI.begin()` 参数为 `sclk=2, miso=3, mosi=3, ss=-1`，与官方 app 一致。
- app：`v2.1.2/DuduClock_Firmware/build-sclk2/DuduClock_Firmware.ino.bin`
- SHA-256：`5e3f6ff3de886dd298124eb2e08f33fbdb2e6976c7f7eee35d13f8a94a0b55e2`
- 烧录结果：成功，esptool 数据哈希校验通过；等待完整断电后的完整界面观察结果。
- 实机结果：用户完整断电重插后，完整 DuduClock 页面正常显示，并出现待扫描的配网二维码。
- 结论：C4 成功。本地源码与依赖可以生成真正驱动该硬件的完整 C++ 固件；此前持续灰白的根因已解决。

### 2026-08-29 18:08:41–18:09:15 CST — 可见 C++ 改动验证 C5

- 源码改动：在 `wifiConfigBySoftAP()` 显示二维码后，用 C++ 绘制 32 像素高洋红色底栏，并显示 `LOCAL C++ / SCLK2`。
- 目的：以持续可见的画面差异证明源码修改能够真实改变设备显示，而非仅复刻官方固件。
- app：`v2.1.2/DuduClock_Firmware/build-custom-banner/DuduClock_Firmware.ino.bin`
- SHA-256：`3a007b4a984353abafbc53d94e77431348d697f5254b1f089497e1992162a9fd`
- 烧录结果：成功，esptool 数据哈希校验通过；等待完整断电后的横幅观察结果。
- 实机结果：用户看到二维码底部新增横幅和文字；横幅在该 TN 屏上呈偏紫/紫粉色，符合 `TFT_MAGENTA`（RGB565 `0xF81F`）的实际观感。
- 结论：C5 成功，已直接证明修改 C++ 源码能够稳定改变实机显示效果。

### 2026-08-29 18:22:07–18:22:47 CST — 最小 Rust 显示验证 R1

- 工程：`esp32c3-display-rust/`，`no_std` + `esp-hal 1.1.2` + `embedded-graphics 0.8`。
- 目标：只验证 Rust 能稳定驱动屏幕显示彩色图像，不实现 DuduClock 业务功能。
- 已修正硬件参数：CPU 80 MHz；SPI Mode 0 / 27 MHz；SCLK=GPIO2、MOSI=GPIO3、CS=GPIO7、DC=GPIO4、RST=GPIO5；GPIO6 不操作。
- 画面：黑色背景、青色边框、`Rust on ESP32-C3` 标题、硬件信息、六段彩条、彩色圆形和橙色动态进度条。
- 烧录结果：成功；芯片识别为 ESP32-C3 rev 0.4 / 4MB，应用占用 `110128 / 4128768` 字节（2.67%）。
- release ELF：`esp32c3-display-rust/target/riscv32imc-unknown-none-elf/release/esp32c3-display-rust`；SHA-256：`0f598c037a10e36c1a8b1bc9c0240c96e9c615d984e712b130231a518eb35cf9`。
- 串口结果：ST7789 初始化、清屏和首次绘制均完成，动态循环从 `Tick #0` 持续运行到至少 `Tick #26`，无崩溃或重启。
- 实机屏幕结果：用户目视确认图像和动态内容均正常，不再灰白。
- 编译环境坑：系统 PATH 优先命中 Homebrew Rust，而 RISC-V target 安装在 rustup 工具链。必须显式让 Cargo 使用 rustup 的 rustc；准确命令已记录在 `esp32c3-display-rust/README.md`。
- 结论：R1 成功。Rust `no_std` 固件可以稳定驱动这块 ST7789V；后续可直接在此工程上用较低成本模型开发功能。

### 2026-08-29 18:16:33 CST — 上游反馈

- GitHub 账号：`yunyingk`。
- 上游仓库限制普通用户创建 issue，因此采用 fork + Pull Request 方式反馈。
- PR：`https://github.com/leezicai/DuduClock_Firmware/pull/2`
- 提交：`49e52cd0c62d5aa4b8af19916813cad050ef3d62`
- 内容：将 `TFT_SCLK` 从 GPIO10 修正为 GPIO2，并在 README 记录本次实测硬件参数、完整引脚、官方固件反汇编证据及不同 PCB/屏幕批次的适用范围提醒。
- 状态：OPEN，非 Draft；上游 main 完整编译检查通过。

> 📝 **给接手开发者（如 ChatGPT / Claude / 人类工程师）的重要备忘**：
> 本文详细记录了在此硬件上尝试 Rust 裸机点屏以及 C++ 源码魔改时**踩过的所有深坑、排查过程、错误根因及终极解法**。请直接阅读下文以避免重复踩坑！

---

## 🎯 一、 硬件与项目基本背景

* **项目名称**：DuduClock（嘟嘟天气时钟 2.1）
* **主控板**：ESP32-C3（RISC-V 32-bit，内置原生 USB CDC / JTAG，macOS 串口设备通常为 `/dev/cu.usbmodem11401`）
* **屏幕模块**：2.4 寸 TFT LCD（驱动 IC: ST7789V，分辨率: 240x320）
* **已验证的出厂固件**：`backup/DuduClock_2.1.2.bin`（刷入 0x0 即可 100% 正常显示）

---

## ⚠️ 二、 踩坑复盘全记录（曾经犯过的所有错误）

### 踩坑 1：引脚配置猜测错误（导致最初屏幕毫无反应）
* **现象**：刚开始开发时，尝试猜测标准 SPI 引脚（4/5/6/7/10 等随意排列），导致屏幕没有任何数据写入。
* **逆向后的真实引脚（必须严格遵守）**：
  * **MOSI (SDA)**: `GPIO 3`
  * **SCLK (SCL)**: `GPIO 2`（官方 2.1.2 app 机器码实证）
  * **CS (片选)**: `GPIO 7`
  * **DC (命令/数据)**: `GPIO 4`
  * **RST (硬件复位)**: `GPIO 5`
  * **BL (背光控制)**: 后续源码版本使用 `GPIO 6` 做 PWM；已知正常的官方 2.1.2 与最小 Rust R1 均不操作该脚
  * **按键 KEY**: `GPIO 8`
  * **I2C 传感器 (AHT20/SHT30)**: SDA=`GPIO 8`, SCL=`GPIO 9`

---

### 踩坑 2：屏幕色彩反相（Inversion）开关会影响观感，但不是本次灰白根因
* **现象**：Rust 驱动（或部分通用 ST7789 驱动库）默认向屏幕发送了 `INVON (0x21)`（色彩反相开启）。
* **后果**：颜色和明暗关系可能与预期不同；但本次反复出现的“只有灰白背光、没有任何图像”的实证根因是 SCLK 错写为 GPIO10，而不是反相命令。
* **正确做法**：必须明确发送 `INVOFF (0x20)`（关闭色彩反转），或者在 `TFT_eSPI` 中定义 `#define TFT_INVERSION_OFF`。

---

### 踩坑 3：缺少电荷泵与偏压初始化序列（国产 2.4 寸 ST7789 面板必须参数）
* **现象**：很多通用的最简驱动只发了 `SLPOUT (0x11)`、`COLMOD (0x3A)` 和 `DISPON (0x29)`，屏幕依然不显示内容。
* **原因**：这块 2.4 寸 10-Pin ST7789V 屏幕的内部电荷泵升压电路需要显式配置门极电压、VCOM 和 Gamma 曲线才能产生足够的液晶偏转电场。
* **必须发送的完整初始化寄存器序列**：
  1. `0x11` (SLPOUT) + 延时 120ms
  2. `0x13` (NORON)
  3. `0x3A` (COLMOD) -> `0x55` (16-bit RGB565)
  4. `0x36` (MADCTL) -> `0x08` (BGR 颜色顺序)
  5. `0xB6` (Display Function) -> `[0x0A, 0x82]`
  6. `0xB0` (RAMCTRL) -> `[0x00, 0xE0]`
  7. `0xB2` (PORCTRL) -> `[0x0C, 0x0C, 0x00, 0x33, 0x33]`
  8. `0xB7` (GCTRL) -> `[0x35]` (VGH / VGL 电压)
  9. `0xBB` (VCOMS) -> `[0x28]` (VCOM 电压)
  10. `0xC0` (LCMCTRL) -> `[0x0C]`
  11. `0xC2` (VDVVRHEN) -> `[0x01, 0xFF]`
  12. `0xC3` (VRHS) -> `[0x10]`
  13. `0xC4` (VDVSET) -> `[0x20]`
  14. `0xC6` (FRCTRL2) -> `[0x0F]` (60Hz 刷新率)
  15. `0xD0` (PWCTRL1) -> `[0xA4, 0xA1]`
  16. `0xE0` (PVGAMCTRL 正极性 Gamma) -> `[0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x32, 0x44, 0x42, 0x06, 0x0E, 0x12, 0x14, 0x17]`
  17. `0xE1` (NVGAMCTRL 负极性 Gamma) -> `[0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x31, 0x54, 0x47, 0x0E, 0x1C, 0x17, 0x1B, 0x1E]`
  18. `0x20` (INVOFF 关闭反相)
  19. `0x29` (DISPON 开启显示) + 延时 120ms

---

### 踩坑 4：Arduino ESP32 Core 3.x 导致 TFT_eSPI 崩溃（Store Access Fault 死机重启）
* **现象**：使用最新的 Arduino ESP32 Core 3.x（如 3.3.11）编译原版 C++ 固件后刷入，板子不断报错崩溃并重启：
  ```text
  Guru Meditation Error: Core 0 panic'ed (Store access fault). Exception was unhandled.
  MEPC: 0x4200c85c (TFT_eSPI.cpp:81 -> SET_BUS_WRITE_MODE)
  ```
* **原因**：
  * 原作者使用的是老版 `TFT_eSPI 2.5.43`，该库直接通过指针写入 ESP32-C3 的底层 SPI 硬件寄存器 `*_spi_user = SPI_USR_MOSI`。
  * 在 ESP32 Arduino Core 3.x 中，Espressif 重构了外设驱动，在调用 `tft.init()` 时，由于 SPI 外设的时钟门控尚未开启，直接访问该寄存器地址会导致内存写保护异常（Store access fault）。
* **解法**：
  * **推荐方案**：精确复刻官方 2.1.2 时统一锁定为 **Arduino-ESP32 Core 2.0.14**（官方 BIN 构建元数据已确认）。
  * 或者若在 3.x 下开发，必须重构 SPI 发送层，使用标准 HAL API 替代裸写寄存器。

---

### 踩坑 5：Flash 分区表容量不足（Sketch too big）
* **现象**：默认 Arduino 分区表只为应用程序预留了 1.2MB（1310720 字节），而 DuduClock 内置了字库点阵和太空人动画数组，编译出的二进制约 1.28MB ~ 3.1MB，编译报 `Sketch too big`。
* **解法**：编译时必须指定分区方案为 **Huge APP**（`PartitionScheme=huge_app`，为应用程序提供 3MB Flash 空间，无 OTA）。

---

### 踩坑 6：背光控制电路的 NMOS 特性（GPIO 6）
* **版本差异**：仓库后续源码把 GPIO6 当作 NMOS 栅极并用 5kHz PWM 调光，但可正常显示的官方 2.1.2 机器码没有操作 GPIO6。
* **最小程序原则**：C3 彩条与 Rust R1 在 GPIO6 完全不操作时均正常显示，因此新显示驱动不要把 GPIO6 当成点屏前提；需要调光时再单独验证。

---

## 🛠️ 三、 给 ChatGPT 的开发建议与极速指引

### 如果你打算用 C/C++ 继续开发：
1. 请直接在 `DuduClock_Firmware/` 目录进行修改。
2. 精确复刻官方 2.1.2 时保持 Arduino ESP32 Core 版本为 `2.0.14`，分区方案选 `huge_app`。
3. 编译与烧录命令：
   ```bash
   arduino-cli compile --fqbn esp32:esp32:esp32c3:CDCOnBoot=cdc,FlashMode=dio,FlashFreq=40,FlashSize=4M,PartitionScheme=huge_app --export-binaries DuduClock_Firmware
   ```

### 如果你打算用 Rust 重新开发最小 Demo：
1. 进入 `esp32c3-display-rust/`。
2. 使用 `esp-hal 1.1.2`，CPU 80MHz，SPI 配置 27MHz Mode 0，引脚严格对应：`SCLK=2`, `MOSI=3`, `CS=7`, `DC=4`, `RST=5`；最小显示版不要操作 GPIO6。
3. 在 SPI 初始化之后，**必须严格按上文【踩坑 3】发送完整的 Gamma 与偏压寄存器参数**，且**必须发送 `0x20 (INVOFF)`**，严禁使用默认 `INVON`。

### 随时恢复正常兜底命令：
```bash
uv run --with esptool esptool --port /dev/cu.usbmodem11401 write-flash 0x0 backup/DuduClock_2.1.2.bin
```
