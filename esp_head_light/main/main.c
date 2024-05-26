/*
 * SPDX-FileCopyrightText: 2010-2022 Espressif Systems (Shanghai) CO LTD
 *
 * SPDX-License-Identifier: CC0-1.0
 */

#include <stdio.h>
#include <inttypes.h>
#include "sdkconfig.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_chip_info.h"
#include "esp_flash.h"
#include "esp_system.h"
#include "esp_wifi.h"
#include "nvs_flash.h"
#include "esp_log.h"
#include "lwip/ip_addr.h"
#include "lwip/err.h"
#include "lwip/sockets.h"
#include "lwip/sys.h"
#include "lwip/udp.h"
#include "mdns.h"
#include "lwip/opt.h"
#include "lwip/arch.h"
#include "lwip/netbuf.h"
#include "lwip/sys.h"
#include "lwip/ip_addr.h"
#include "lwip/err.h"
#include "lwip/api.h"

void resolve_mdns_host(const char *host_name)
{
    printf("Query A: %s.local\n", host_name);

    esp_ip4_addr_t addr;
    addr.addr = 0;

    esp_err_t err = mdns_query_a(host_name, 2000, &addr);
    if (err)
    {
        if (err == ESP_ERR_NOT_FOUND)
        {
            printf("Host was not found!\n");
            return;
        }
        printf("Query Failed\n");
        return;
    }

    printf(IPSTR, IP2STR(&addr));
}

static EventGroupHandle_t s_wifi_event_group;
const char *ssid = "";
const char *pass = "";
static int s_retry_num = 0;
#define WIFI_CONNECTED_BIT BIT0
#define WIFI_FAIL_BIT BIT1
static void event_handler(void *arg, esp_event_base_t event_base,
                          int32_t event_id, void *event_data)
{
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START)
    {
        esp_wifi_connect();
    }
    else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED)
    {
        if (s_retry_num < 100)
        {
            esp_wifi_connect();
            s_retry_num++;
            ESP_LOGI("MAIN_WIFI", "retry to connect to the AP");
        }
        else
        {
            xEventGroupSetBits(s_wifi_event_group, WIFI_FAIL_BIT);
        }
        ESP_LOGI("MAIN_WIFI", "connect to the AP fail");
    }
    else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP)
    {
        ip_event_got_ip_t *event = (ip_event_got_ip_t *)event_data;
        ESP_LOGI("MAIN_WIFI", "got ip:" IPSTR, IP2STR(&event->ip_info.ip));
        s_retry_num = 0;
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    }
}
static const char *ip_protocol_str[] = {"V4", "V6", "MAX"};

void mdns_print_results(mdns_result_t *results)
{
    mdns_result_t *r = results;
    mdns_ip_addr_t *a = NULL;
    int i = 1, t;
    while (r)
    {
        printf("%d: Type: %s\n", i++, ip_protocol_str[r->ip_protocol]);
        if (r->instance_name)
        {
            printf("  PTR : %s\n", r->instance_name);
        }
        if (r->hostname)
        {
            printf("  SRV : %s.local:%u\n", r->hostname, r->port);
        }
        if (r->txt_count)
        {
            printf("  TXT : [%u] ", r->txt_count);
            for (t = 0; t < r->txt_count; t++)
            {
                printf("%s=%s; ", r->txt[t].key, r->txt[t].value);
            }
            printf("\n");
        }
        a = r->addr;
        while (a)
        {
            if (a->addr.type == IPADDR_TYPE_V6)
            {
                printf("  AAAA: " IPV6STR "\n", IPV62STR(a->addr.u_addr.ip6));
            }
            else
            {
                printf("  A   : " IPSTR "\n", IP2STR(&(a->addr.u_addr.ip4)));
                // struct udp_pcb *udp_sock = udp_new();
                // udp_bind(udp_sock, IP_ADDR_ANY, 0);
                // struct sockaddr_in addr;
                // addr.sin_addr.s_addr = a->addr.u_addr.ip4.addr;
                // addr.sin_family = AF_INET;
                // addr.sin_port = htons(r->port);
                // udp_sendto(udp_sock, "Hello", 5, (struct sockaddr *)&addr, r->port);

                struct netconn_t *conn = netconn_new(NETCONN_TCP);
                if (conn != NULL)
                {
                    err_t err = netconn_connect(conn, &a->addr.u_addr.ip4, r->port);
                    if (err == ERR_OK)
                    {
                        printf("Connected to %s:%d\n", ip4addr_ntoa(&a->addr.u_addr.ip4), r->port);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_write(conn, "Hello", 5, NETCONN_COPY);
                        netconn_close(conn);
                        netconn_delete(conn);
                    }
                    else
                    {
                        printf("Connection failed\n");
                    }
                }
                else
                {
                    printf("Failed to create new connection\n");
                }
            }
            a = a->next;
        }
        r = r->next;
    }
}
void find_mdns_service(const char *service_name, const char *proto)
{
    ESP_LOGI("MAIN_MDNS", "Query PTR: %s.%s.local", service_name, proto);

    mdns_result_t *results = NULL;
    esp_err_t err = mdns_query_ptr(service_name, proto, 3000, 20, &results);

    printf("################ Query result: %d\n", err);
    if (err)
    {
        ESP_LOGE("MAIN_MDNS", "Query Failed");
        return;
    }
    if (!results)
    {
        ESP_LOGW("MAIN_MDNS", "No results found!");
        return;
    }

    mdns_print_results(results);
    mdns_query_results_free(results);
}

void app_main(void)
{
    printf("Hello world!\n");
    ESP_ERROR_CHECK(nvs_flash_init());
    s_wifi_event_group = xEventGroupCreate();
    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());

    /* Print chip information */
    esp_chip_info_t chip_info;
    uint32_t flash_size;
    esp_chip_info(&chip_info);
    printf("This is %s chip with %d CPU core(s), %s%s%s%s, ",
           CONFIG_IDF_TARGET,
           chip_info.cores,
           (chip_info.features & CHIP_FEATURE_WIFI_BGN) ? "WiFi/" : "",
           (chip_info.features & CHIP_FEATURE_BT) ? "BT" : "",
           (chip_info.features & CHIP_FEATURE_BLE) ? "BLE" : "",
           (chip_info.features & CHIP_FEATURE_IEEE802154) ? ", 802.15.4 (Zigbee/Thread)" : "");

    unsigned major_rev = chip_info.revision / 100;
    unsigned minor_rev = chip_info.revision % 100;
    printf("silicon revision v%d.%d, ", major_rev, minor_rev);
    if (esp_flash_get_size(NULL, &flash_size) != ESP_OK)
    {
        printf("Get flash size failed");
        return;
    }

    printf("%" PRIu32 "MB %s flash\n", flash_size / (uint32_t)(1024 * 1024),
           (chip_info.features & CHIP_FEATURE_EMB_FLASH) ? "embedded" : "external");

    esp_netif_t *wifi = esp_netif_create_default_wifi_sta();
    wifi_init_config_t wifi_initialization = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&wifi_initialization));
    esp_event_handler_instance_t instance_any_id;
    esp_event_handler_instance_t instance_got_ip;
    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &event_handler, NULL, &instance_any_id));
    ESP_ERROR_CHECK(esp_event_handler_instance_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &event_handler, NULL, &instance_got_ip));
    wifi_config_t wifi_configuration = {
        .sta = {
            .ssid = "D7D433",
            .password = "8RaycXmTcHcG",
            .threshold.authmode = WIFI_AUTH_WPA_WPA2_PSK,
        },
    };
    ESP_ERROR_CHECK(esp_netif_dhcpc_start(wifi));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_configuration));
    ESP_ERROR_CHECK(esp_wifi_start());

    EventBits_t bits = xEventGroupWaitBits(s_wifi_event_group,
                                           WIFI_CONNECTED_BIT | WIFI_FAIL_BIT,
                                           pdFALSE,
                                           pdFALSE,
                                           portMAX_DELAY);

    printf("wifi_init_softap finished. SSID:%s\n", "D7D443");
    if (bits & WIFI_CONNECTED_BIT)
    {
        ESP_LOGI("MAIN_WIFI", "connected to ap SSID:D7D443");
    }
    else if (bits & WIFI_FAIL_BIT)
    {
        ESP_LOGI("MAIN_WIFI", "Failed to connect to SSID:D7D443");
    }
    else
    {
        ESP_LOGE("MAIN_WIFI", "UNEXPECTED EVENT");
    }
    mdns_init();
    printf("Minimum free heap size: %" PRIu32 " bytes\n", esp_get_minimum_free_heap_size());
    for (int i = 10; i >= 0; i--)
    {

        // resolve_mdns_host("RtAudioEffect");
        // resolve_mdns_host("RtAudioEffect.local");
        // resolve_mdns_host("RtAudioEffect.local.");
        // resolve_mdns_host("_RtAudioEffect._udp");
        // resolve_mdns_host("_RtAudioEffect._udp.local");
        // resolve_mdns_host("_RtAudioEffect._udp.local.");

        find_mdns_service("_RtAudioEffect", "_tcp");
        printf("Restarting in %d seconds...\n", i);
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }
    printf("Restarting now.\n");
    fflush(stdout);
    esp_restart();
}
