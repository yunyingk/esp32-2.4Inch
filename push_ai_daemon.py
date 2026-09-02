#!/usr/bin/env python3
"""
DuduClock AI Monitor Daemon
在 Mac 后台循环运行，定时向 ESP32-C3 小屏幕打请求，保持 AI 余额与运行状态常驻。
"""

import sys
import time
import urllib.request
import json

DEFAULT_ESP_IP = "192.168.3.188"
INTERVAL_SECONDS = 30  # 每 30 秒打一次心跳请求，保证 180s 租期始终满血

def push_to_esp(ip: str, title: str, quota: str, sub_info: str, lease_seconds: int = 180):
    url = f"http://{ip}/api/display"
    payload = {
        "title": title,
        "quota": quota,
        "sub_info": sub_info,
        "status": "ONLINE",
        "lease_seconds": lease_seconds,
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=3) as resp:
            return resp.status == 200
    except Exception as e:
        print(f"[{time.strftime('%H:%M:%S')}] 推送失败 ({url}): {e}")
        return False

def main():
    esp_ip = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_ESP_IP
    print(f"==================================================")
    print(f" DuduClock AI 监控常驻后台守护进程启动")
    print(f" 目标设备: http://{esp_ip}/api/display")
    print(f" 刷新间隔: {INTERVAL_SECONDS} 秒")
    print(f" 按 Ctrl+C 退出 (退出后小屏幕将在倒计时归零后自动回退到时钟)")
    print(f"==================================================")

    count = 1
    while True:
        title = "Antigravity AI"
        quota = "$28.45"
        sub_info = f"Uptime: {count * INTERVAL_SECONDS}s | Status: Normal"
        
        ok = push_to_esp(esp_ip, title, quota, sub_info, lease_seconds=180)
        if ok:
            print(f"[{time.strftime('%H:%M:%S')}] 成功向 {esp_ip} 推送 AI 状态 (# {count})")
        
        count += 1
        time.sleep(INTERVAL_SECONDS)

if __name__ == "__main__":
    main()
