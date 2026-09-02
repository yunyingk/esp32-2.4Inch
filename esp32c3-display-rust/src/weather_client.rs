// DuduClock 互联网天气模块 (no_std)

#[derive(Clone, Debug)]
pub struct WeatherData {
    pub text: heapless::String<32>,
    pub temp: heapless::String<16>,
    pub humidity: heapless::String<16>,
    pub display_str: heapless::String<32>,
}

impl Default for WeatherData {
    fn default() -> Self {
        let mut display_str = heapless::String::new();
        let _ = display_str.push_str("晴 28℃ / 55%");
        Self {
            text: heapless::String::new(),
            temp: heapless::String::new(),
            humidity: heapless::String::new(),
            display_str,
        }
    }
}

impl WeatherData {
    pub fn update_from_str(&mut self, text: &str) {
        self.display_str.clear();
        let trimmed = text.trim();
        let clean = if trimmed.is_empty() { "28°C / 50%" } else { trimmed };
        let _ = self.display_str.push_str(clean);
    }
}
