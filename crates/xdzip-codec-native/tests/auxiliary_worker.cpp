// Authored protocol fixture. No archive decoder or compressed symbols.
#include "protocol.hpp"
#include <array>

namespace xdzip {
static void fake_run() {
    uint8_t mode = 0;
    if (read_source(7, &mode, 1) != 1)
        throw Failure{2};
    if (request.operation == 0) {
        if (mode == 'u' || mode == 'n' || mode == 'w') {
            metadata(0, L"unknown.bin", mode == 'w' ? 0x80 : 0x40, mode == 'n' ? 1 : 0, 1, 0, 29);
            return;
        }
        if (mode == 's') {
            std::vector<uint8_t> frame;
            put64(frame, 1);
            send(AuxWork, frame);
            ack();
        }
        metadata(0, L"owned.bin", 0, 87, 1, 0, 29);
        return;
    }
    auto forged = [](uint64_t count, bool tail) {
        std::vector<uint8_t> frame;
        put64(frame, count);
        if (tail)
            frame.push_back(99);
        send(AuxWork, frame);
        ack();
    };
    if (mode == 'e') {
        send(AuxWork);
        ack();
        return;
    }
    if (mode == 'z') {
        forged(0, false);
        return;
    }
    if (mode == 'o') {
        forged(65537, false);
        return;
    }
    if (mode == 't') {
        forged(1, true);
        return;
    }
    if (mode == 'f') {
        forged(1, false);
        return;
    }
    if (mode == 'q') {
        for (uint64_t n = 0; n <= (20ULL << 30) / 65536; ++n)
            charge_auxiliary(65536);
        throw Failure{2}; // The host must reject before this point.
    }
    for (unsigned n = 0; n != 81; ++n)
        charge_auxiliary(65536);
    if (mode == 'r' && request.password != "retry")
        throw Failure{7};
    std::array<uint8_t, 87> data{};
    for (unsigned n = 0; n != data.size(); ++n)
        data[n] = static_cast<uint8_t>(n);
    if (mode != 'p')
        write_data(data.data(), data.size(), false);
    write_precharged_data(data.data(), data.size());
}
void rar_run() {
    fake_run();
}
void seven_run() {
    fake_run();
}
void readonly_run() {
    fake_run();
}
} // namespace xdzip
