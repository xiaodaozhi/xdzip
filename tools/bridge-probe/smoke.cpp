// Diagnostic boundary checks stay active in release builds as well as debug.
#include "rust/cxx.h"
#include "xdzip-bridge-probe/src/lib.rs.h"
#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>
#include <thread>
#ifdef XDZIP_PROBE_QT
#include <QString>
#endif

namespace {
void require(bool condition, const char *message) {
    if (!condition) {
        throw std::runtime_error(message);
    }
}

template <typename Operation> void require_error(Operation operation) {
    try {
        operation();
    } catch (const rust::Error &) {
        return;
    }
    throw std::runtime_error("expected a translated Rust error");
}

void check_boundary() {
    using namespace xdzip::probe;
    require_error([] { new_probe(0); });
    require_error([] { new_probe(129); });
    auto probe = new_probe(2);
    auto client = probe->take_client();
    auto receiver = probe->take_receiver();
    require_error([&] { probe->take_client(); });
    require_error([&] { probe->take_receiver(); });
    std::string original = u8"目录/压缩📦/é";
    original.append(1, '\0');
    original += "tail";
#ifdef XDZIP_PROBE_QT
    const auto qt_input =
        QString::fromUtf8(original.data(), static_cast<qsizetype>(original.size()));
    const auto qt_bytes = qt_input.toUtf8();
    std::string input(qt_bytes.constData(), static_cast<std::size_t>(qt_bytes.size()));
#else
    std::string input = original;
#endif
    const auto first = client->submit(rust::Str(input.data(), input.size()));
    input.assign("changed after submit");
    require(first.status == Admission::Accepted && first.request_id == 1, "first admission");
    require(client->submit("").status == Admission::Accepted, "empty payload admission");
    require(client->submit("full").status == Admission::Busy, "bounded queue");
    const std::string oversized(4097, 'x');
    require(client->submit(oversized).status == Admission::TooLarge, "payload limit");
    client->shutdown();
    client->shutdown();
    require(client->submit("late").status == Admission::Closed, "closed admission");
    auto event = receiver->next_event();
    require(!event.end && event.request_id == 1, "ordered event");
    require(std::string(event.payload.data(), event.payload.size()) == original,
            "owned UTF-8 bytes");
#ifdef XDZIP_PROBE_QT
    require(QString::fromUtf8(event.payload.data(), static_cast<qsizetype>(event.payload.size())) ==
                qt_input,
            "Qt UTF-8 roundtrip");
#endif
    auto empty = receiver->next_event();
    require(!empty.end && empty.request_id == 2 && empty.payload.empty(), "empty is not end");
    require(receiver->next_event().end && receiver->next_event().end, "idempotent end");
}

void check_thread_shutdown() {
    auto probe = xdzip::probe::new_probe(1);
    auto client = probe->take_client();
    auto receiver = probe->take_receiver();
    bool ended = false;
    // Move unique ownership to the receiver thread; no C++ alias crosses threads.
    std::thread worker(
        [receiver = std::move(receiver), &ended]() mutable { ended = receiver->next_event().end; });
    client->shutdown();
    worker.join();
    require(ended, "shutdown must wake receiver, including shutdown-before-wait");
}
} // namespace

int main() {
    try {
        check_boundary();
        check_thread_shutdown();
        std::cout << "bridge ownership, errors, UTF-8, bounds and shutdown: PASS\n";
        return 0;
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
