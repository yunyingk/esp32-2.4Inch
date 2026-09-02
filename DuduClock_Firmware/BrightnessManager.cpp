#include "BrightnessManager.h"

BrightnessManager::BrightnessManager(uint8_t minBrightness, uint8_t maxBrightness)
    : _minBrightness(minBrightness),
      _maxBrightness(maxBrightness),
      _brightSamplingTime(0),
      _brightSamplingValue(0),
      _currentBrightness(minBrightness),
      _rawLightValue(0),
      _lastSampleTime(0),
      _lastCalculationTime(0) {
}

void BrightnessManager::init() {
    // 配置 ADC 为更高分辨率 (12 位)
    // Configure ADC for higher resolution (12-bit)
    analogReadResolution(12);
    
    
    
    // 初始化变量
    // Initialize variables
    _brightSamplingTime = 0;
    _brightSamplingValue = 0;
    _lastSampleTime = 0;
    _lastCalculationTime = 0;
    
    // 获取初始读数
    // Take initial readings
    for (int i = 0; i < 5; i++) {
        _brightSamplingValue += analogRead(LIGHT_ADC);
        _brightSamplingTime++;
        delay(10); // 读数之间的短暂延迟 Short delay between readings
    }
    
    // 计算初始亮度
    // Calculate initial brightness
    calculateBrightnessValue();
    
    // Serial.print("BrightnessManager initialized with brightness: ");
    // Serial.println(_currentBrightness);
}

void BrightnessManager::handle() {
    // 处理亮度采样和计算
    // Handle brightness sampling and calculation
    handleBrightnessSampling();
}

void BrightnessManager::handleBrightnessSampling() {
    // 按固定间隔采集样本
    // Take samples at regular intervals
    if (millis() - _lastSampleTime >= SAMPLING_INTERVAL) {
        // 从光线传感器读取 ADC 值
        // Read ADC value from light sensor
        int sensorValue = analogRead(LIGHT_ADC);
        
        // 详细调试信息
        // Detailed debug information
        // Serial.print("Raw ADC reading: ");
        // Serial.print(sensorValue);
        // Serial.print(" (");
        // Serial.print((float)sensorValue / 4095.0 * 3.3);
        // Serial.println("V)");
        
        // 累加样本
        // Accumulate sample
        _brightSamplingValue += sensorValue;
        _brightSamplingTime++;
        
        // 更新上次采样时间
        // Update last sample time
        _lastSampleTime = millis();
    }
    
    // 每秒计算一次亮度
    // Calculate brightness once per second
    if (millis() - _lastCalculationTime >= CALCULATION_INTERVAL) {
        calculateBrightnessValue();
        _lastCalculationTime = millis();
    }
}

void BrightnessManager::calculateBrightnessValue() {
    // Calculate average from samples
    int val = 0;
    if (_brightSamplingTime > 0) {
        val = _brightSamplingValue / _brightSamplingTime;
    }
    
    // Save raw light sensor value
    _rawLightValue = val;
    
    // Apply the new brightness logic
    if (val >= 3000) {
        // Map full ADC range (0-4095) to brightness range (0-255)
        int mappedBrightness = map(val, 3100, 4095, 100, 255);
        
        // Constrain to the configured brightness range
        _currentBrightness = constrain(mappedBrightness, _minBrightness, _maxBrightness);
    } else {
        // Low light environment, set brightness to 0
        _currentBrightness = 0;
    }
    
    // Print debug information
    // Serial.print("Sample count: ");
    // Serial.print(_brightSamplingTime);
    // Serial.print(", Average value: ");
    // Serial.print(val);
    // Serial.print(", Current brightness: ");
    // Serial.println(_currentBrightness);
    
    // Reset counters for next calculation cycle
    _brightSamplingTime = 0;
    _brightSamplingValue = 0;
}

uint8_t BrightnessManager::getCurrentBrightness() const {
    return _currentBrightness;
}

void BrightnessManager::setBrightnessRange(uint8_t minBrightness, uint8_t maxBrightness) {
    _minBrightness = minBrightness;
    _maxBrightness = maxBrightness;
    
    // Serial.print("Brightness range updated: ");
    // Serial.print(_minBrightness);
    // Serial.print(" - ");
    // Serial.println(_maxBrightness);
}

uint8_t BrightnessManager::getMinBrightness() const {
    return _minBrightness;
}

uint8_t BrightnessManager::getMaxBrightness() const {
    return _maxBrightness;
}

int BrightnessManager::getRawLightValue() const {
    return _rawLightValue;
}