## 问题

当前 `local_lib/User_Setup.h` 将 TFT SPI 时钟配置为 GPIO 10。在本次测试的
DuduClock 2.1 实机上，本地编译固件可以正常启动、扫描 Wi-Fi 并运行 Web 配网服务，
但屏幕只亮背光，画面始终灰白。

## 适用硬件与范围

本次实机参数：

- ESP32-C3 rev 0.4
- ST7789V
- SPI
- 240×320
- 显示区域 36.72×48.96 mm
- TN 12 o'clock

不同 PCB 或屏幕批次可能采用不同走线，因此 README 中保留了核对硬件版本的提示。

## 验证证据

下载的官方 v2.1.2 完整固件在同一台设备上正常显示。其 app 反汇编显示，
`TFT_eSPI::init()` 调用 `SPIClass::begin(sclk, miso, mosi, ss)` 前加载的参数为：

```text
li a4, -1
li a3, 3
li a2, 3
li a1, 2
jal SPIClass::begin
```

即 `SPI.begin(2, 3, 3, -1)`，SCLK 实际为 GPIO 2。

实机 A/B 结果：

1. 官方 v2.1.2 完整固件及官方 app-only：正常显示。
2. 源码配置 SCLK=GPIO10：完整断电重插后仍始终灰白，串口和网络正常。
3. 最小 TFT_eSPI 彩条程序使用 GPIO10：始终灰白。
4. 同一最小程序只将 SCLK 改为 GPIO2：红、绿、蓝、白、黑和彩条正常循环。
5. 完整 v2.1.2 源码使用 GPIO2：配网二维码正常显示；加入自定义 C++ 横幅后实机也正常显示。

## 修改

- 将 `TFT_SCLK` 从 GPIO10 修正为 GPIO2。
- 在 README 记录已验证硬件参数、完整 TFT 引脚和不同硬件批次的注意事项。

## 构建验证

使用以下环境完整编译通过：

- Arduino-ESP32 2.0.14
- AirM2M_CORE_ESP32C3
- CPU 80 MHz / Flash 80 MHz
- Huge APP
- TFT_eSPI 2.5.43

