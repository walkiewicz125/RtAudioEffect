#pragma once
#include <cstdint>
#include <vector>
#include <string>

// wire format:
// [id: u32 | len: u32 | payload: u8[len]]

struct Message
{
    virtual std::vector<std::uint8_t> &&serialize() = 0;
};

struct EchoMessage : public Message
{
    EchoMessage(std::string message)
    {
        data.resize(sizeof(std::uint32_t) + sizeof(std::uint32_t) + message.size());

        std::uint32_t *id = reinterpret_cast<std::uint32_t *>(data.data());
        std::uint32_t *len = reinterpret_cast<std::uint32_t *>(data.data() + sizeof(std::uint32_t));
        std::uint8_t *payload = data.data() + sizeof(std::uint32_t) + sizeof(std::uint32_t);

        *id = EchoMessage::id;
        *len = message.size();
        std::copy(message.begin(), message.end(), payload);
    }

    std::vector<std::uint8_t> &&serialize() override
    {
        return std::move(data);
    }

    static constexpr std::uint32_t id = 1;

private:
    std::vector<std::uint8_t> data;
};
