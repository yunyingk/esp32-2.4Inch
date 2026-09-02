#![no_std]
#![no_main]

extern crate alloc;

mod ble_service;
mod chinese_font;
mod config;
mod http_server;
mod ntp_client;
mod ui_pages;
mod weather_client;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
};
use esp_backtrace as _;
use esp_wifi::ble::controller::BleConnector;
use crate::ble_service::BleService;

#[unsafe(export_name = "esp_app_desc")]
#[unsafe(link_section = ".rodata_desc")]
#[used]
pub static ESP_APP_DESC: esp_bootloader_esp_idf::EspAppDesc =
    esp_bootloader_esp_idf::EspAppDesc::new_internal(
        "1.0.0",
        "duduclock",
        "18:50:00",
        "2026-08-29",
        "5.1",
        0,          // min_efuse_blk_rev_full (0 -> support v0.4 / v1.3 chip)
        u16::MAX,   // max_efuse_blk_rev_full
        64 * 1024,  // mmu_page_size (64KB for ESP32-C3)
        0,          // secure_version
    );

use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
    rng::Rng,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

use crate::{
    ui_pages::{render_ai_quota, render_standby_clock, AiQuotaData},
    weather_client::WeatherData,
};

pub struct St7789Display<'a> {
    spi: Spi<'a, esp_hal::Blocking>,
    cs: Output<'a>,
    dc: Output<'a>,
    rst: Output<'a>,
}

impl<'a> St7789Display<'a> {
    pub fn new(
        spi: Spi<'a, esp_hal::Blocking>,
        cs: Output<'a>,
        dc: Output<'a>,
        rst: Output<'a>,
    ) -> Self {
        Self { spi, cs, dc, rst }
    }

    #[inline(always)]
    fn write_cmd(&mut self, cmd: u8, data: &[u8]) {
        self.cs.set_level(Level::Low);
        self.dc.set_level(Level::Low);
        let _ = self.spi.write(&[cmd]);
        if !data.is_empty() {
            self.dc.set_level(Level::High);
            let _ = self.spi.write(data);
        }
        self.cs.set_level(Level::High);
    }

    pub fn init(&mut self, delay: &Delay) {
        // 硬件复位
        self.rst.set_level(Level::High);
        delay.delay_millis(20);
        self.rst.set_level(Level::Low);
        delay.delay_millis(50);
        self.rst.set_level(Level::High);
        delay.delay_millis(150);

        // 1. Sleep Out
        self.write_cmd(0x11, &[]);
        delay.delay_millis(120);

        // 2. Normal Display On
        self.write_cmd(0x13, &[]);

        // 3. Color Format (RGB565 16-bit)
        self.write_cmd(0x3A, &[0x55]);
        delay.delay_millis(10);

        // 4. Memory Access Control (MADCTL: MX=1, MV=1, BGR=1 -> 0x68 = 90 deg clockwise landscape 320x240)
        self.write_cmd(0x36, &[0x68]);

        // 5. Display function / Ram control
        self.write_cmd(0xB6, &[0x0A, 0x82]);
        self.write_cmd(0xB0, &[0x00, 0xE0]);

        // 6. Porch Control (PORCTRL)
        self.write_cmd(0xB2, &[0x0C, 0x0C, 0x00, 0x33, 0x33]);

        // 7. Gate Control (GCTRL - VGH/VGL voltages)
        self.write_cmd(0xB7, &[0x35]);

        // 8. VCOM Setting (VCOMS)
        self.write_cmd(0xBB, &[0x28]);

        // 9. LCM Control
        self.write_cmd(0xC0, &[0x0C]);

        // 10. VDV and VRH command enable
        self.write_cmd(0xC2, &[0x01, 0xFF]);

        // 11. VRH Set
        self.write_cmd(0xC3, &[0x10]);

        // 12. VDV Set
        self.write_cmd(0xC4, &[0x20]);

        // 13. Frame Rate Control in Normal Mode (FRCTRL2 = 60Hz)
        self.write_cmd(0xC6, &[0x0F]);

        // 14. Power Control 1 (PWCTRL1)
        self.write_cmd(0xD0, &[0xA4, 0xA1]);

        // 15. Positive Voltage Gamma Control (PVGAMCTRL)
        self.write_cmd(
            0xE0,
            &[
                0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x32, 0x44, 0x42, 0x06, 0x0E, 0x12, 0x14, 0x17,
            ],
        );

        // 16. Negative Voltage Gamma Control (NVGAMCTRL)
        self.write_cmd(
            0xE1,
            &[
                0xD0, 0x00, 0x02, 0x07, 0x0A, 0x28, 0x31, 0x54, 0x47, 0x0E, 0x1C, 0x17, 0x1B, 0x1E,
            ],
        );

        // 17. Inversion Off (0x20 - TFT_INVERSION_OFF)
        self.write_cmd(0x20, &[]);

        // 18. Set default address window 0..319 x 0..239
        self.set_window(0, 0, 319, 239);

        // 19. Display On
        delay.delay_millis(50);
        self.write_cmd(0x29, &[]);
        delay.delay_millis(120);
    }

    pub fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
        self.write_cmd(
            0x2A,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xFF) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xFF) as u8,
            ],
        );
        self.write_cmd(
            0x2B,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xFF) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xFF) as u8,
            ],
        );
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: Rgb565) {
        if w == 0 || h == 0 {
            return;
        }
        self.set_window(x, y, x + w - 1, y + h - 1);
        self.cs.set_level(Level::Low);
        self.dc.set_level(Level::Low);
        let _ = self.spi.write(&[0x2C]);
        self.dc.set_level(Level::High);

        let color_raw = color.into_storage();
        let high = (color_raw >> 8) as u8;
        let low = (color_raw & 0xFF) as u8;

        let mut buf = [0u8; 480];
        let total_pixels = (w as usize) * (h as usize);
        for i in (0..480).step_by(2) {
            buf[i] = high;
            buf[i + 1] = low;
        }

        let mut remaining = total_pixels;
        while remaining > 0 {
            let chunk_pixels = if remaining > 240 { 240 } else { remaining };
            let _ = self.spi.write(&buf[0..chunk_pixels * 2]);
            remaining -= chunk_pixels;
        }
        self.cs.set_level(Level::High);
    }

    pub fn clear(&mut self, color: Rgb565) {
        self.fill_rect(0, 0, 320, 240, color);
    }
}

impl<'a> OriginDimensions for St7789Display<'a> {
    fn size(&self) -> Size {
        Size::new(320, 240)
    }
}

impl<'a> DrawTarget for St7789Display<'a> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x >= 0 && x < 320 && y >= 0 && y < 240 {
                self.set_window(x as u16, y as u16, x as u16, y as u16);
                self.cs.set_level(Level::Low);
                self.dc.set_level(Level::Low);
                let _ = self.spi.write(&[0x2C]);
                self.dc.set_level(Level::High);
                let raw = color.into_storage();
                let _ = self.spi.write(&[(raw >> 8) as u8, (raw & 0xFF) as u8]);
                self.cs.set_level(Level::High);
            }
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let intersection = area.intersection(&Rectangle::new(Point::zero(), self.size()));
        if intersection.size.width > 0 && intersection.size.height > 0 {
            self.fill_rect(
                intersection.top_left.x as u16,
                intersection.top_left.y as u16,
                intersection.size.width as u16,
                intersection.size.height as u16,
                color,
            );
        }
        Ok(())
    }
}

#[derive(PartialEq)]
enum DisplayMode {
    StandbyClock,
    AiQuotaMonitor,
}

#[main]
fn main() -> ! {
    // 1. 初始化 140KB 堆内存 (支持 Wi-Fi + BLE 共存)
    esp_alloc::heap_allocator!(size: 140 * 1024);

    // 开启 RISC-V 全局机器中断 (MIE bit 3 in mstatus)
    unsafe {
        core::arch::asm!("csrsi mstatus, 8");
    }

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);
    let delay = Delay::new();

    println!("==========================================");
    println!("  DuduClock ESP32-C3 Wi-Fi & AI Receiver  ");
    println!("==========================================");

    // 2. 初始化 ST7789 屏幕 (320x240 横屏)。GPIO6 保持复位态，
    //    该硬件版本主动驱动背光脚可能导致灰白或亮度异常。
    let sclk = peripherals.GPIO2;
    let mosi = peripherals.GPIO3;
    let cs = Output::new(peripherals.GPIO7, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());

    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(27))
        .with_mode(Mode::_0);

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .unwrap()
        .with_sck(sclk)
        .with_mosi(mosi);

    let mut display = St7789Display::new(spi, cs, dc, rst);
    display.init(&delay);
    display.clear(Rgb565::BLACK);

    // 3. 初始化 BLE 蓝牙协议栈 (Pure BLE 模式，专供外出无 Wi-Fi 场景)
    let timer_group = TimerGroup::new(peripherals.TIMG0);
    let init = esp_wifi::init(timer_group.timer0, Rng::new(peripherals.RNG)).unwrap();

    println!("[Init] Initializing BLE Bluetooth controller...");
    let ble_connector = BleConnector::new(&init, peripherals.BT);
    let mut ble_service = BleService::new(ble_connector);

    println!("[Init] Starting BLE Bluetooth advertising ('{}')...", config::BLE_DEVICE_NAME);
    let ble_ok = ble_service.start_advertising(&delay, config::BLE_DEVICE_NAME);
    println!("[Init] BLE Bluetooth ready: {} ('{}')", ble_ok, config::BLE_DEVICE_NAME);

    // 4. 状态变量
    let mut current_mode = DisplayMode::StandbyClock;
    let mut ai_data = AiQuotaData::default();
    let mut weather = WeatherData::default();
    let mut remaining_lease_secs: u32 = 0;
    let mut total_lease_secs: u32 = config::DEFAULT_LEASE_SECONDS;

    let mut ip_str: heapless::String<20> = heapless::String::new();
    let _ = ip_str.push_str("BLE 蓝牙已在线");

    let mut date_str: heapless::String<32> = heapless::String::new();
    let _ = date_str.push_str("2026-08-29 星期六");

    // 真实北京时钟
    let mut hours: u8 = 19;
    let mut mins: u8 = 48;
    let mut secs: u8 = 0;
    let mut last_second_tick: u64 = 0;

    println!("DuduClock BLE Ready! Waiting for Bluetooth data...");

    // 首次绘制待机时钟页面
    render_standby_clock(
        &mut display,
        &ip_str,
        hours,
        mins,
        secs,
        &date_str,
        &weather.display_str,
    );

    let mut ble_rx_buf = [0u8; 256];

    loop {
        let now_micros = esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64;
        let now_millis = (now_micros / 1000) as u64;

        // 轮询 BLE 蓝牙数据
        let ble_len = ble_service.poll_packet(&mut ble_rx_buf);
        if ble_len > 0 {
            println!("BLE packet received: {} bytes", ble_len);
            if let Ok(text) = core::str::from_utf8(&ble_rx_buf[..ble_len]) {
                println!("BLE Text: {}", text);
                if http_server::parse_json_data(text, &mut ai_data) {
                    println!("BLE Updated AI Quota Data: {:?}", ai_data);
                    total_lease_secs = ai_data.lease_seconds;
                    remaining_lease_secs = total_lease_secs;
                    current_mode = DisplayMode::AiQuotaMonitor;
                    render_ai_quota(
                        &mut display,
                        &ip_str,
                        &ai_data,
                        remaining_lease_secs,
                        total_lease_secs,
                    );
                }
            }
        }

        // 每秒时间递增与界面刷新逻辑
        if now_millis >= last_second_tick + 1000 {
            last_second_tick = now_millis;

            if secs % 5 == 0 {
                println!("[Heartbeat] BLE Advertising: Live, Device: 'Dudu-AI-Screen', Mode: {}", if current_mode == DisplayMode::AiQuotaMonitor { "AI Monitor" } else { "Clock" });
            }

            // 秒递增
            secs += 1;
            if secs >= 60 {
                secs = 0;
                mins += 1;
                if mins >= 60 {
                    mins = 0;
                    hours = (hours + 1) % 24;
                }
            }

            // 倒计时与模式切换
            if current_mode == DisplayMode::AiQuotaMonitor {
                if remaining_lease_secs > 0 {
                    remaining_lease_secs -= 1;
                    render_ai_quota(
                        &mut display,
                        &ip_str,
                        &ai_data,
                        remaining_lease_secs,
                        total_lease_secs,
                    );
                } else {
                    println!("AI Quota display lease expired, switching to Standby Clock.");
                    current_mode = DisplayMode::StandbyClock;
                    render_standby_clock(
                        &mut display,
                        &ip_str,
                        hours,
                        mins,
                        secs,
                        &date_str,
                        &weather.display_str,
                    );
                }
            } else {
                render_standby_clock(
                    &mut display,
                    &ip_str,
                    hours,
                    mins,
                    secs,
                    &date_str,
                    &weather.display_str,
                );
            }
        }

        delay.delay_millis(2);
    }
}
