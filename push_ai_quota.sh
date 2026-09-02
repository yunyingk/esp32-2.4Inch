#!/bin/bash
# DuduClock AI 余额与状态快速推送脚本 (Mac 端)
# 用法: ./push_ai_quota.sh [ESP32_IP] [TITLE] [QUOTA] [SUB_INFO] [LEASE_SECS]

ESP_IP="${1:-192.168.3.188}"
TITLE="${2:-Claude 3.7 Sonnet}"
QUOTA="${3:-\$18.42}"
SUB_INFO="${4:-Today: \$1.20 | Tokens: 1.4M}"
LEASE_SECS="${5:-180}"

echo "=================================================="
echo " 向 DuduClock 发送 AI 监控数据..."
echo " 目标设备: http://${ESP_IP}/api/display"
echo " 模型标题: ${TITLE}"
echo " 剩余余额: ${QUOTA}"
echo " 详细信息: ${SUB_INFO}"
echo " 保持时间: ${LEASE_SECS} 秒"
echo "=================================================="

curl -s -X POST "http://${ESP_IP}/api/display" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"${TITLE}\",
    \"quota\": \"${QUOTA}\",
    \"sub_info\": \"${SUB_INFO}\",
    \"status\": \"ACTIVE\",
    \"lease_seconds\": ${LEASE_SECS}
  }"

echo -e "\n\n[OK] 数据已发送！屏幕应立即切换至 AI 监控大屏并启动 ${LEASE_SECS} 秒倒计时。"
