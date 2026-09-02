// DuduClock UI 渲染模块 (320x240 横屏 - 中文优化版)

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10, FONT_9X15_BOLD},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::Text,
};

use crate::chinese_font::draw_chinese_str;

/// AI 监控数据结构
#[derive(Clone, Debug)]
pub struct AiQuotaData {
    pub title: heapless::String<32>,
    pub quota: heapless::String<32>,
    pub sub_info: heapless::String<48>,
    pub status: heapless::String<16>,
    pub lease_seconds: u32,
}

impl Default for AiQuotaData {
    fn default() -> Self {
        let mut title = heapless::String::new();
        let _ = title.push_str("AI 模型监控");
        let mut quota = heapless::String::new();
        let _ = quota.push_str("$0.00");
        let mut sub_info = heapless::String::new();
        let _ = sub_info.push_str("等待电脑推送数据...");
        let mut status = heapless::String::new();
        let _ = status.push_str("在线");
        Self {
            title,
            quota,
            sub_info,
            status,
            lease_seconds: 180,
        }
    }
}

/// 渲染 待机时钟与天气屏 (Mode B)
pub fn render_standby_clock<D>(
    display: &mut D,
    ip_str: &str,
    hours: u8,
    mins: u8,
    secs: u8,
    date_str: &str,
    temp_str: &str,
) where
    D: DrawTarget<Color = Rgb565>,
{
    // 1. 全屏深色背景
    let _ = display.clear(Rgb565::new(0, 2, 4));

    // 2. 外边框 (深青蓝)
    let _ = Rectangle::new(Point::new(2, 2), Size::new(316, 236))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::new(4, 16, 24))
                .stroke_width(2)
                .stroke_alignment(StrokeAlignment::Inside)
                .build(),
        )
        .draw(display);

    // 3. 顶部状态栏
    let _ = Rectangle::new(Point::new(10, 8), Size::new(300, 28))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(1, 4, 8))
                .stroke_color(Rgb565::new(6, 20, 30))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    // 中文日期与星期 (例如 2026-08-29 星期六)
    draw_chinese_str(display, 16, 14, date_str, Rgb565::CYAN);

    // Wi-Fi 状态绿点与 IP
    let _ = Circle::new(Point::new(200, 16), 8)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(display);
    let ip_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let _ = Text::new(ip_str, Point::new(214, 26), ip_style).draw(display);

    // 4. 核心超大时钟数字卡片 (深黑底 + 霓虹金黄文字)
    let _ = Rectangle::new(Point::new(10, 42), Size::new(300, 94))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(2, 3, 6))
                .stroke_color(Rgb565::new(10, 24, 36))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    // 格式化时间 HH:MM:SS
    let mut time_str: heapless::String<16> = heapless::String::new();
    let _ = core::fmt::write(
        &mut time_str,
        format_args!("{:02}:{:02}:{:02}", hours, mins, secs),
    );

    let time_style = MonoTextStyle::new(&FONT_10X20, Rgb565::new(31, 56, 12)); // 金黄色
    let _ = Text::new(&time_str, Point::new(88, 96), time_style).draw(display);

    draw_chinese_str(display, 110, 112, "桌面待机时钟", Rgb565::new(18, 42, 10));

    // 5. 下方两个信息卡片 (天气/温度 + 系统状态)
    // 左卡片: 天气与温度
    let _ = Rectangle::new(Point::new(10, 142), Size::new(145, 54))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(2, 4, 8))
                .stroke_color(Rgb565::new(8, 16, 24))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    draw_chinese_str(display, 18, 148, "天气与温度", Rgb565::CSS_GRAY);
    draw_chinese_str(display, 18, 172, temp_str, Rgb565::WHITE);

    // 右卡片: AI 接收端状态
    let _ = Rectangle::new(Point::new(165, 142), Size::new(145, 54))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(2, 4, 8))
                .stroke_color(Rgb565::new(8, 16, 24))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    draw_chinese_str(display, 173, 148, "监控接收端", Rgb565::CSS_GRAY);
    draw_chinese_str(display, 173, 172, "在线待命", Rgb565::GREEN);

    // 6. 底部提示栏
    let _ = Rectangle::new(Point::new(10, 202), Size::new(300, 28))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(1, 3, 6))
                .stroke_color(Rgb565::new(4, 12, 18))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    let tip_style = MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW);
    let mut tip_msg: heapless::String<64> = heapless::String::new();
    let _ = core::fmt::write(
        &mut tip_msg,
        format_args!("POST http://{}/api/display", ip_str),
    );
    let _ = Text::new(&tip_msg, Point::new(20, 220), tip_style).draw(display);
}

/// 渲染 AI 余额与状态大屏 (Mode A)
pub fn render_ai_quota<D>(
    display: &mut D,
    ip_str: &str,
    data: &AiQuotaData,
    remaining_secs: u32,
    total_lease_secs: u32,
) where
    D: DrawTarget<Color = Rgb565>,
{
    // 1. 全屏深紫黑渐变底色
    let _ = display.clear(Rgb565::new(1, 1, 3));

    // 2. 科技感外边框 (洋红/青色)
    let _ = Rectangle::new(Point::new(2, 2), Size::new(316, 236))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::MAGENTA)
                .stroke_width(2)
                .stroke_alignment(StrokeAlignment::Inside)
                .build(),
        )
        .draw(display);

    // 3. 顶部标题栏
    let _ = Rectangle::new(Point::new(10, 8), Size::new(300, 30))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(3, 1, 6))
                .stroke_color(Rgb565::new(18, 4, 24))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    draw_chinese_str(display, 18, 15, &data.title, Rgb565::CYAN);

    // 绿点状态
    let _ = Circle::new(Point::new(240, 18), 8)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
        .draw(display);
    draw_chinese_str(display, 254, 15, "在线", Rgb565::GREEN);

    // 4. 核心超大余额展示区域
    let _ = Rectangle::new(Point::new(10, 44), Size::new(300, 92))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(4, 2, 8))
                .stroke_color(Rgb565::new(20, 6, 28))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    draw_chinese_str(display, 18, 50, "当前模型剩余额度", Rgb565::CSS_GRAY);

    // 大数值
    let quota_style = MonoTextStyle::new(&FONT_10X20, Rgb565::CSS_ORANGE);
    let _ = Text::new(&data.quota, Point::new(32, 104), quota_style).draw(display);

    // 状态标签
    draw_chinese_str(display, 240, 90, &data.status, Rgb565::YELLOW);

    // 5. 中间副信息指标栏
    let _ = Rectangle::new(Point::new(10, 142), Size::new(300, 36))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(2, 2, 5))
                .stroke_color(Rgb565::new(10, 10, 20))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    draw_chinese_str(display, 18, 152, &data.sub_info, Rgb565::WHITE);

    // 6. 底部倒计时租期条 (带动态进度条)
    let _ = Rectangle::new(Point::new(10, 184), Size::new(300, 46))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::new(2, 2, 4))
                .stroke_color(Rgb565::new(8, 8, 16))
                .stroke_width(1)
                .build(),
        )
        .draw(display);

    // 进度条背景槽
    let _ = Rectangle::new(Point::new(18, 192), Size::new(284, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::new(4, 4, 8)))
        .draw(display);

    // 实际进度
    let total = if total_lease_secs == 0 { 180 } else { total_lease_secs };
    let progress_width = ((remaining_secs * 284) / total).min(284);
    if progress_width > 0 {
        let _ = Rectangle::new(Point::new(18, 192), Size::new(progress_width, 10))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_ORANGE))
            .draw(display);
    }

    // 倒计时文字提示
    let mins = remaining_secs / 60;
    let secs = remaining_secs % 60;
    let mut cd_str: heapless::String<64> = heapless::String::new();
    let _ = core::fmt::write(
        &mut cd_str,
        format_args!("倒计时 {:02}:{:02} 后返回时钟", mins, secs),
    );
    draw_chinese_str(display, 18, 208, &cd_str, Rgb565::CYAN);
}
