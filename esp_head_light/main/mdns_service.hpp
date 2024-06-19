#pragma once

#include "mdns.h"
#include <lwip/ip_addr.h>
#include <lwip/api.h>

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

#include "message/message.hpp"

static const char *ip_protocol_str[] = {"V4", "V6", "MAX"};

struct ServiceAddr
{
    uint32_t addr;
    uint16_t port;

    bool is_valid() const
    {
        return addr != 0 && port != 0;
    }
};

struct MdnsService
{
    MdnsService(const char *service_name, const char *proto)
        : service_name(service_name), proto(proto)
    {
    }

    ~MdnsService()
    {
        if (connection != nullptr)
        {
            netconn_close(connection);
            netconn_delete(connection);
        }
    }

    bool find_service()
    {
        printf("Querying mDNS for %s:%s\n", service_name, proto);

        mdns_result_t *results = find_mdns_service(service_name, proto);
        mdns_print_results(results);
        addres_found = get_address(results);
        log_address(addres_found);
        mdns_query_results_free(results);

        return addres_found.is_valid();
    }

    bool connect()
    {
        connection = netconn_new(NETCONN_TCP);
        if (connection != nullptr)
        {
            ip4_addr_t addr4{addres_found.addr};
            err_t err = netconn_connect(connection, reinterpret_cast<const ip_addr_t *>(&addr4), addres_found.port);
            if (err == ERR_OK)
            {
                printf("Connected to %s:%d\n", ip4addr_ntoa(&addr4), addres_found.port);
                return true;
            }
            else
            {
                printf("Connection failed\n");
            }
        }
        else
        {
            printf("Failed to create connection\n");
        }

        return false;
    }

    bool send(Message &message)
    {
        auto data = message.serialize();
        if (netconn_write(connection, data.data(), data.size(), NETCONN_COPY) != ERR_OK)
        {
            printf("Failed to send message\n");
            return false;
        }

        return true;
    }

private:
    ServiceAddr addres_found;
    const char *service_name;
    const char *proto;
    netconn *connection;

    void log_address(const ServiceAddr &addr)
    {
        if (addr.is_valid())
        {
            printf("Found headlight service at %s:%u\n", ip4addr_ntoa(reinterpret_cast<const ip4_addr_t *>(&addr.addr)), addr.port);
        }
        else
        {
            printf("Headlight service not found\n");
        }
    }

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
                }
                a = a->next;
            }
            r = r->next;
        }
    }

    ServiceAddr get_address(mdns_result_t *results)
    {
        mdns_result_t *r = results;
        mdns_ip_addr_t *a = NULL;
        while (r)
        {
            a = r->addr;
            while (a)
            {
                if (a->addr.type == IPADDR_TYPE_V4)
                {
                    return {a->addr.u_addr.ip4.addr, r->port};
                }
                a = a->next;
            }
            r = r->next;
        }
        return {0, 0};
    }

    mdns_result_t *find_mdns_service(const char *service_name, const char *proto)
    {
        ESP_LOGI("MAIN_MDNS", "Query PTR: %s.%s.local", service_name, proto);

        mdns_result_t *results = nullptr;
        esp_err_t err = mdns_query_ptr(service_name, proto, 3000, 20, &results);

        printf("################ Query result: %d\n", err);
        if (err)
        {
            ESP_LOGE("MAIN_MDNS", "Query Failed");
            return nullptr;
        }
        if (!results)
        {
            ESP_LOGW("MAIN_MDNS", "No results found!");
            return nullptr;
        }

        return results;
    }
};
