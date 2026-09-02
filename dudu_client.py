#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
DuduClock 智能桌面副屏客户端 (Dudu Client)
功能：
1. 自动发现局域网中的 DuduClock 小屏幕设备 (无需手动输入 IP，自动穿透代理/VPN虚拟网卡)
2. 封装数据包并一键推送 AI 余额、模型名称、状态提示到小屏幕
3. 支持单次推送、命令行参数推送与定时监控推送
"""

import sys
import json
import time
import socket
import argparse
import subprocess
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor

DEFAULT_TIMEOUT = 0.5
PORT = 80

def get_physical_subnets():
    """获取本机所有物理 Wi-Fi / 局域网网段 (自动过滤 Clash 198.18.x 和 Tailscale 100.x)"""
    subnets = []
    try:
        out = subprocess.check_output(["ifconfig"], text=True)
        for line in out.splitlines():
            line = line.strip()
            if line.startswith("inet "):
                parts = line.split()
                ip = parts[1]
                # 过滤本地回环、Tailscale、Clash 虚拟 IP
                if ip.startswith("127.") or ip.startswith("198.18.") or ip.startswith("100."):
                    continue
                prefix = ".".join(ip.split(".")[:3]) + "."
                if prefix not in subnets:
                    subnets.append(prefix)
    except Exception:
        pass
    
    if not subnets:
        subnets = ["192.168.3.", "192.168.1.", "192.168.0."]
    return subnets

def probe_dudu_screen(ip):
    """探测单个 IP 是否为 DuduClock 设备"""
    url = f"http://{ip}/api/display"
    try:
        req = urllib.request.Request(
            url,
            data=json.dumps({"probe": True}).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=DEFAULT_TIMEOUT) as resp:
            if resp.status == 200:
                return ip
    except Exception:
        pass
    return None

def scan_for_dudu_screen(custom_ip=None):
    """扫描物理局域网寻找 DuduClock 设备"""
    if custom_ip:
        if probe_dudu_screen(custom_ip):
            return custom_ip

    subnets = get_physical_subnets()
    
    # 优先快速测试已知常见 IP
    for prefix in subnets:
        for fast_ip in [f"{prefix}61", f"{prefix}188", f"{prefix}100"]:
            if probe_dudu_screen(fast_ip):
                print(f"✨ 快速命中 DuduClock 设备: {fast_ip}")
                return fast_ip

    for prefix in subnets:
        print(f"🔍 正在扫描物理网段 ({prefix}1 ~ {prefix}254)...")
        with ThreadPoolExecutor(max_workers=60) as executor:
            futures = {executor.submit(probe_dudu_screen, f"{prefix}{i}"): f"{prefix}{i}" for i in range(1, 255)}
            for future in futures:
                res = future.result()
                if res:
                    print(f"✅ 成功发现 DuduClock 设备: {res}")
                    return res
                
    return None

def send_payload(target_ip, title, quota, detail="", lease=180):
    """向小屏幕发送数据封装包"""
    url = f"http://{target_ip}/api/display"
    payload = {
        "title": title,
        "quota": quota,
        "detail": detail,
        "lease": lease
    }
    data = json.dumps(payload).encode("utf-8")
    
    print("=" * 55)
    print(f"🚀 发送数据包到 DuduClock ({url})")
    print(f"   模型名称: {title}")
    print(f"   剩余余额: {quota}")
    print(f"   状态详情: {detail}")
    print(f"   保持时间: {lease} 秒")
    print("=" * 55)
    
    try:
        req = urllib.request.Request(
            url,
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=3) as resp:
            resp_body = resp.read().decode("utf-8")
            print(f"🎉 推送成功！设备响应: {resp_body.strip()}")
            return True
    except Exception as e:
        print(f"❌ 发送失败: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description="DuduClock 智能桌面副屏专用客户端")
    parser.add_argument("--ip", help="指定小屏幕 IP 地址 (默认自动扫描发现)")
    parser.add_argument("--title", default="Claude 3.7 Sonnet", help="模型标题 (默认: Claude 3.7 Sonnet)")
    parser.add_argument("--quota", default="$35.00", help="余额额度 (默认: $35.00)")
    parser.add_argument("--detail", default="专属客户端智能管道 | 自动发现", help="详细信息")
    parser.add_argument("--lease", type=int, default=180, help="屏幕显示保持时间(秒)")
    parser.add_argument("--scan-only", action="store_true", help="仅扫描寻找设备")
    
    args = parser.parse_args()
    
    target_ip = scan_for_dudu_screen(args.ip)
    if not target_ip:
        print("❌ 未在局域网内找到 DuduClock 设备。请确认小屏幕已开机。")
        sys.exit(1)
            
    if args.scan_only:
        print(f"🎯 找到设备 IP: {target_ip}")
        return

    send_payload(target_ip, args.title, args.quota, args.detail, args.lease)

if __name__ == "__main__":
    main()
