// DuduClock HTTP REST 请求解析模块 (no_std)

use crate::ui_pages::AiQuotaData;

/// 从简易 JSON 字符串中提取指定 key 的值
fn extract_json_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let key_pattern_quote = format_pattern_quote(key);
    let key_idx = find_substr(json, &key_pattern_quote)?;
    let after_key = &json[key_idx + key_pattern_quote.len()..];
    
    // 跳过冒号和空白
    let mut chars = after_key.char_indices();
    let mut in_val = false;
    let mut val_start = 0;
    
    for (idx, c) in chars.by_ref() {
        if c == '"' {
            if !in_val {
                in_val = true;
                val_start = idx + 1;
            } else {
                return Some(&after_key[val_start..idx]);
            }
        } else if !in_val && (c.is_ascii_digit() || c == '.' || c == '$' || c == '-') {
            // 非引号数字/字符串
            val_start = idx;
            let mut end_idx = idx;
            for (sub_idx, sub_c) in after_key[val_start..].char_indices() {
                if sub_c == ',' || sub_c == '}' || sub_c.is_whitespace() {
                    end_idx = val_start + sub_idx;
                    break;
                }
            }
            return Some(&after_key[val_start..end_idx]);
        }
    }
    None
}

fn format_pattern_quote(key: &str) -> heapless::String<32> {
    let mut s = heapless::String::new();
    let _ = s.push('"');
    let _ = s.push_str(key);
    let _ = s.push('"');
    let _ = s.push(':');
    s
}

fn find_substr(haystack: &str, needle: &str) -> Option<usize> {
    let h_bytes = haystack.as_bytes();
    let n_bytes = needle.as_bytes();
    if n_bytes.is_empty() || n_bytes.len() > h_bytes.len() {
        return None;
    }
    for i in 0..=h_bytes.len() - n_bytes.len() {
        if &h_bytes[i..i + n_bytes.len()] == n_bytes {
            return Some(i);
        }
    }
    None
}

/// 直接解析 JSON 字符串或 BLE 数据包
pub fn parse_json_data(body: &str, target: &mut AiQuotaData) -> bool {
    let mut updated = false;

    if let Some(t) = extract_json_str(body, "title") {
        target.title.clear();
        let _ = target.title.push_str(t);
        updated = true;
    }

    if let Some(q) = extract_json_str(body, "quota") {
        target.quota.clear();
        let _ = target.quota.push_str(q);
        updated = true;
    } else if let Some(b) = extract_json_str(body, "balance") {
        target.quota.clear();
        let _ = target.quota.push_str(b);
        updated = true;
    }

    if let Some(s) = extract_json_str(body, "sub_info") {
        target.sub_info.clear();
        let _ = target.sub_info.push_str(s);
        updated = true;
    } else if let Some(msg) = extract_json_str(body, "msg") {
        target.sub_info.clear();
        let _ = target.sub_info.push_str(msg);
        updated = true;
    } else if let Some(d) = extract_json_str(body, "detail") {
        target.sub_info.clear();
        let _ = target.sub_info.push_str(d);
        updated = true;
    }

    if let Some(st) = extract_json_str(body, "status") {
        target.status.clear();
        let _ = target.status.push_str(st);
        updated = true;
    }

    if let Some(lease_str) = extract_json_str(body, "lease_seconds") {
        if let Ok(secs) = lease_str.trim().parse::<u32>() {
            if secs > 0 {
                target.lease_seconds = secs;
                updated = true;
            }
        }
    } else if let Some(lease_str) = extract_json_str(body, "lease") {
        if let Ok(secs) = lease_str.trim().parse::<u32>() {
            if secs > 0 {
                target.lease_seconds = secs;
                updated = true;
            }
        }
    }

    updated
}

/// 解析 HTTP 请求数据包并更新 AiQuotaData
pub fn parse_http_display_request(request_str: &str, target: &mut AiQuotaData) -> bool {
    let body = if let Some(idx) = find_substr(request_str, "\r\n\r\n") {
        &request_str[idx + 4..]
    } else if let Some(idx) = find_substr(request_str, "\n\n") {
        &request_str[idx + 2..]
    } else {
        request_str
    };
    parse_json_data(body, target)
}

/// 解析天气推送请求 (POST /api/weather)
pub fn parse_http_weather_request(request_str: &str, weather_out: &mut heapless::String<32>) -> bool {
    let body = if let Some(idx) = find_substr(request_str, "\r\n\r\n") {
        &request_str[idx + 4..]
    } else if let Some(idx) = find_substr(request_str, "\n\n") {
        &request_str[idx + 2..]
    } else {
        request_str
    };

    if let Some(w) = extract_json_str(body, "weather") {
        weather_out.clear();
        let _ = weather_out.push_str(w);
        return true;
    }
    if let Some(t) = extract_json_str(body, "temp") {
        weather_out.clear();
        let _ = weather_out.push_str(t);
        return true;
    }
    false
}
