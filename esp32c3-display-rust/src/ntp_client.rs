// DuduClock NTP 北京时间同步模块 (no_std)

pub const ALIYUN_NTP_IP: [u8; 4] = [203, 107, 6, 88]; // ntp.aliyun.com
pub const NTP_PORT: u16 = 123;

#[derive(Clone, Copy, Debug)]
pub struct BeijingTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub weekday: &'static str,
}

impl Default for BeijingTime {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 8,
            day: 29,
            hour: 19,
            minute: 0,
            second: 0,
            weekday: "SAT",
        }
    }
}

/// 构建标准 NTP 客户端 48 字节请求数据包
pub fn create_ntp_request() -> [u8; 48] {
    let mut packet = [0u8; 48];
    // LI = 0 (no warning), VN = 3 (NTP v3), Mode = 3 (Client) -> 0x1B
    packet[0] = 0x1B;
    packet
}

/// 解析 NTP 响应数据包并转换为北京时间 (UTC+8)
pub fn parse_ntp_response(packet: &[u8]) -> Option<BeijingTime> {
    if packet.len() < 48 {
        return None;
    }

    // Transmit Timestamp Seconds (Bytes 40..44)
    let sec_1900 = u32::from_be_bytes([packet[40], packet[41], packet[42], packet[43]]);
    if sec_1900 < 2208988800 {
        return None;
    }

    // 转换为 1970 Unix Timestamp + 8 小时 (UTC+8)
    let unix_ts = (sec_1900 - 2208988800) as u64;
    let beijing_ts = unix_ts + 8 * 3600;

    let sec = (beijing_ts % 60) as u8;
    let minute = ((beijing_ts / 60) % 60) as u8;
    let hour = ((beijing_ts / 3600) % 24) as u8;

    // 简易公历日期推算 (从 1970-01-01 开始)
    let mut total_days = (beijing_ts / 86400) as u32;
    // 1970-01-01 是星期四 (4)
    let weekday_idx = (total_days + 4) % 7;
    let weekdays = ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"];
    let weekday = weekdays[weekday_idx as usize];

    let mut year: u16 = 1970;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if total_days >= days_in_year {
            total_days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let days_in_months = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month: u8 = 1;
    for &m_days in &days_in_months {
        if total_days >= m_days {
            total_days -= m_days;
            month += 1;
        } else {
            break;
        }
    }
    let day = (total_days + 1) as u8;

    Some(BeijingTime {
        year,
        month,
        day,
        hour,
        minute,
        second: sec,
        weekday,
    })
}
