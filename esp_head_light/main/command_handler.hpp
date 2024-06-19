#include "mdns_service.hpp"
#include "message/message.hpp"

struct CommandHandler
{
    CommandHandler(MdnsService &service)
        : service(service)
    {
    }

    bool test_connection()
    {
        printf("Testing connection\n");
        EchoMessage echo("Hello, world!");

        printf("Minimum free heap size: %" PRIu32 " bytes\n", esp_get_minimum_free_heap_size());
        service.send(echo);
        return false;
    }

private:
    MdnsService &service;
};
