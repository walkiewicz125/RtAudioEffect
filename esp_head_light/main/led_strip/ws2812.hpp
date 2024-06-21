#pragma once

#include <stdlib.h>
#include <string.h>
#include <sys/cdefs.h>
#include "esp_log.h"
#include "esp_attr.h"
#include "led_strip.h"
#include "driver/rmt.h"

#include <vector>

#define WS2812_T0H_NS (350)
#define WS2812_T0L_NS (1000)
#define WS2812_T1H_NS (1000)
#define WS2812_T1L_NS (350)
#define WS2812_RESET_US (280)

#ifdef __cplusplus
extern "C"
{
#endif

    static uint32_t ws2812_t0h_ticks = 0;
    static uint32_t ws2812_t1h_ticks = 0;
    static uint32_t ws2812_t0l_ticks = 0;
    static uint32_t ws2812_t1l_ticks = 0;

#ifdef __cplusplus
}
#endif

class Ws2812
{
    Ws2812(rmt_channel_t rmt_channel, size_t led_count)
        : rmt_channel(rmt_channel),
          rgb_buffer(led_count * 3)
    {
    }

private:
    rmt_channel_t rmt_channel;
    std::vector<uint8_t> rgb_buffer;
};
