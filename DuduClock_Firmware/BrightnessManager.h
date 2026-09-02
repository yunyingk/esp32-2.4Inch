#ifndef BRIGHTNESS_MANAGER_H
#define BRIGHTNESS_MANAGER_H

#include <Arduino.h>

// 配置参数
// Configuration parameters
#define LIGHT_ADC 2                // 光线传感器 ADC 引脚 (GPIO 1) Light sensor ADC pin
#define SAMPLING_INTERVAL 100       // 采样间隔（毫秒） Sampling interval (milliseconds)
#define CALCULATION_INTERVAL 1000   // 计算间隔（毫秒） Calculation interval (milliseconds)

/**
 * @brief 管理亮度计算的类，包括自动亮度检测
 * Class that manages brightness calculation, including auto-brightness detection
 */
class BrightnessManager {
public:
    /**
     * @brief BrightnessManager 的构造函数
     * Constructor for BrightnessManager
     * 
     * @param minBrightness 最小亮度值 Minimum brightness value
     * @param maxBrightness 最大亮度值 Maximum brightness value
     */
    BrightnessManager(uint8_t minBrightness = 1, uint8_t maxBrightness = 255);
    
    /**
     * @brief 初始化亮度管理器
     * Initialize the brightness manager
     */
    void init();
    
    /**
     * @brief 处理亮度计算的主函数，应在主循环中调用
     * Main function for brightness calculation, should be called in the main loop
     */
    void handle();
    
    /**
     * @brief 获取当前计算的亮度值
     * Gets the current calculated brightness value
     * 
     * @return uint8_t 当前亮度值 Current brightness value
     */
    uint8_t getCurrentBrightness() const;
    
    /**
     * @brief 设置亮度范围
     * Sets brightness range
     * 
     * @param minBrightness 最小亮度值 Minimum brightness value
     * @param maxBrightness 最大亮度值 Maximum brightness value
     */
    void setBrightnessRange(uint8_t minBrightness, uint8_t maxBrightness);
    
    /**
     * @brief 获取最小亮度值
     * Gets minimum brightness value
     * 
     * @return uint8_t 最小亮度值 Minimum brightness value
     */
    uint8_t getMinBrightness() const;
    
    /**
     * @brief 获取最大亮度值
     * Gets maximum brightness value
     * 
     * @return uint8_t 最大亮度值 Maximum brightness value
     */
    uint8_t getMaxBrightness() const;
    
    /**
     * @brief 获取当前光线传感器原始值
     * Gets current light sensor raw value
     * 
     * @return int 光线传感器原始值 Light sensor raw value
     */
    int getRawLightValue() const;

private:
    // 亮度范围设置
    // Brightness range settings
    uint8_t _minBrightness;         // 最小亮度值 Minimum brightness value
    uint8_t _maxBrightness;         // 最大亮度值 Maximum brightness value
    
    // 亮度计算相关成员
    // Brightness calculation related members
    int _brightSamplingTime;        // 亮度样本数量 Number of brightness samples
    int _brightSamplingValue;       // 累积样本值 Accumulated sample value
    uint8_t _currentBrightness;     // 当前亮度值 Current brightness value
    int _rawLightValue;             // 当前光线传感器原始值 Current light sensor raw value
    unsigned long _lastSampleTime;      // 上次采样时间 Last sample time
    unsigned long _lastCalculationTime; // 上次计算时间 Last calculation time
    
    /**
     * @brief 从累积样本计算亮度值
     * Calculate brightness value from accumulated samples
     */
    void calculateBrightnessValue();
    
    /**
     * @brief 处理自动亮度采样
     * Handle auto brightness sampling
     */
    void handleBrightnessSampling();
};

#endif // BRIGHTNESS_MANAGER_H