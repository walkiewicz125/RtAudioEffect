#include "main.hpp"
#include "mdns_service.hpp"
#include "command_handler.hpp"

void main_cpp()
{
    MdnsService service("_RtAudioEffect", "_tcp");

    if (not service.find_service())
    {
        printf("Service not found\n");
        return;
    }

    if (not service.connect())
    {
        printf("Couldn't connect to service\n");
        return;
    }

    printf("connected to service\n");

    CommandHandler handler(service);
    handler.test_connection();
}
