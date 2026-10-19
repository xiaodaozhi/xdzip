// Read-only OS counters for our explicitly supplied product PID and descendants.
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <thread>
#include <vector>
#ifdef __APPLE__
#include <libproc.h>
#include <mach/mach_time.h>
#include <sys/resource.h>
#elif defined(_WIN32)
#define NOMINMAX
#include <windows.h>

#include <psapi.h>
#include <tlhelp32.h>
#else
#include <filesystem>
#include <unistd.h>
#endif

struct Counters {
    uint64_t start = 0, rss = 0, physical = 0, read = 0, written = 0;
    double cpu = 0;
};

static std::map<int, int> parents() {
    std::map<int, int> result;
#ifdef __APPLE__
    const int bytes = proc_listpids(PROC_ALL_PIDS, 0, nullptr, 0);
    if (bytes <= 0)
        return result;
    std::vector<pid_t> pids(static_cast<size_t>(bytes) / sizeof(pid_t) + 256);
    const int count =
        proc_listpids(PROC_ALL_PIDS, 0, pids.data(), static_cast<int>(pids.size() * sizeof(pid_t)));
    for (int i = 0; i < count / static_cast<int>(sizeof(pid_t)); ++i) {
        proc_bsdinfo info{};
        if (pids[i] > 0 && proc_pidinfo(pids[i], PROC_PIDTBSDINFO, 0, &info, sizeof(info)) ==
                               static_cast<int>(sizeof(info)))
            result[pids[i]] = static_cast<int>(info.pbi_ppid);
    }
#elif defined(_WIN32)
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE)
        return result;
    PROCESSENTRY32 entry{};
    entry.dwSize = sizeof(entry);
    if (Process32First(snapshot, &entry)) {
        do {
            result[static_cast<int>(entry.th32ProcessID)] =
                static_cast<int>(entry.th32ParentProcessID);
        } while (Process32Next(snapshot, &entry));
    }
    CloseHandle(snapshot);
#else
    for (const auto &entry : std::filesystem::directory_iterator("/proc")) {
        const auto name = entry.path().filename().string();
        if (name.empty() || name.find_first_not_of("0123456789") != std::string::npos)
            continue;
        std::ifstream file(entry.path() / "stat");
        std::string line;
        if (!std::getline(file, line))
            continue;
        const auto end = line.rfind(')');
        if (end == std::string::npos)
            continue;
        std::istringstream fields(line.substr(end + 2));
        char state;
        int parent;
        if (fields >> state >> parent)
            result[std::stoi(name)] = parent;
    }
#endif
    return result;
}

static bool readCounters(int pid, Counters &c) {
#ifdef __APPLE__
    rusage_info_v4 info{};
    if (proc_pid_rusage(pid, RUSAGE_INFO_V4, reinterpret_cast<rusage_info_t *>(&info)) != 0)
        return false;
    c.start = info.ri_proc_start_abstime;
    c.rss = info.ri_resident_size;
    c.physical = info.ri_phys_footprint;
    c.read = info.ri_diskio_bytesread;
    c.written = info.ri_diskio_byteswritten;
    // libproc CPU time is Mach absolute ticks, not nanoseconds on Apple Silicon.
    // Keep the same host timebase as mach_absolute_time; failure invalidates the sample.
    static const auto timebase = [] {
        mach_timebase_info_data_t value{};
        if (mach_timebase_info(&value) != KERN_SUCCESS)
            value.denom = 0;
        return value;
    }();
    if (timebase.denom == 0)
        return false;
    c.cpu = (static_cast<double>(info.ri_user_time) + static_cast<double>(info.ri_system_time)) *
            timebase.numer / timebase.denom / 1e9;
#elif defined(_WIN32)
    HANDLE process =
        OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, static_cast<DWORD>(pid));
    if (!process)
        return false;
    PROCESS_MEMORY_COUNTERS_EX memory{};
    IO_COUNTERS io{};
    FILETIME created{}, exited{}, kernel{}, user{};
    const bool ok =
        GetProcessMemoryInfo(process, reinterpret_cast<PROCESS_MEMORY_COUNTERS *>(&memory),
                             sizeof(memory)) &&
        GetProcessIoCounters(process, &io) &&
        GetProcessTimes(process, &created, &exited, &kernel, &user);
    CloseHandle(process);
    if (!ok)
        return false;
    const auto ticks = [](FILETIME value) {
        return (static_cast<uint64_t>(value.dwHighDateTime) << 32) | value.dwLowDateTime;
    };
    c.start = ticks(created);
    c.rss = memory.WorkingSetSize;
    // Private committed bytes are deliberately not labelled physical footprint.
    c.physical = memory.PrivateUsage;
    c.read = io.ReadTransferCount;
    c.written = io.WriteTransferCount;
    c.cpu = static_cast<double>(ticks(kernel) + ticks(user)) / 1e7;
#else
    const std::string base = "/proc/" + std::to_string(pid) + "/";
    std::ifstream stat(base + "stat");
    std::string line;
    if (!std::getline(stat, line) || line.rfind(')') == std::string::npos)
        return false;
    std::istringstream fields(line.substr(line.rfind(')') + 2));
    std::vector<std::string> values;
    std::string value;
    while (fields >> value)
        values.push_back(value);
    if (values.size() < 22)
        return false;
    c.start = std::stoull(values[19]);
    c.cpu = static_cast<double>(std::stoull(values[11]) + std::stoull(values[12])) /
            static_cast<double>(sysconf(_SC_CLK_TCK));
    c.rss = std::stoull(values[21]) * static_cast<uint64_t>(sysconf(_SC_PAGESIZE));
    std::ifstream smaps(base + "smaps_rollup");
    while (std::getline(smaps, line)) {
        if (line.rfind("Pss:", 0) == 0)
            c.physical = std::stoull(line.substr(4)) * 1024;
    }
    std::ifstream io(base + "io");
    while (std::getline(io, line)) {
        if (line.rfind("read_bytes:", 0) == 0)
            c.read = std::stoull(line.substr(11));
        if (line.rfind("write_bytes:", 0) == 0)
            c.written = std::stoull(line.substr(12));
    }
    if (!smaps.eof() || !io.eof() || c.physical == 0)
        return false;
#endif
    return true;
}

int main(int argc, char **argv) {
    if (argc != 3)
        return 2;
    const int root = std::atoi(argv[1]), interval = std::atoi(argv[2]);
    if (root <= 0 || interval < 20 || interval > 1000)
        return 2;
#ifdef __APPLE__
    const char *memoryKind = "physical_footprint", *ioKind = "diskio_bytes";
#elif defined(_WIN32)
    const char *memoryKind = "private_commit", *ioKind = "process_io_transfer_bytes";
#else
    const char *memoryKind = "pss", *ioKind = "storage_read_write_bytes";
#endif
    const auto start = std::chrono::steady_clock::now();
    uint64_t rootStart = 0;
    for (;;) {
        Counters rootCounters;
        if (!readCounters(root, rootCounters))
            break;
        if (rootStart != 0 && rootStart != rootCounters.start)
            break;
        rootStart = rootCounters.start;
        auto tree = parents();
        if (!tree.count(root))
            break;
        std::vector<int> selected{root};
        for (size_t i = 0; i < selected.size(); ++i) {
            for (const auto &[pid, parent] : tree) {
                if (parent == selected[i] &&
                    std::find(selected.begin(), selected.end(), pid) == selected.end())
                    selected.push_back(pid);
            }
        }
        const double elapsed =
            std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start)
                .count();
        std::cout << "{\"elapsed_ms\":" << elapsed << ",\"memory_kind\":\"" << memoryKind
                  << "\",\"io_kind\":\"" << ioKind << "\",\"processes\":[";
        bool first = true;
        for (int pid : selected) {
            Counters c;
            if (!readCounters(pid, c))
                continue;
            if (!first)
                std::cout << ',';
            first = false;
            std::cout << "{\"pid\":" << pid << ",\"start\":\"" << c.start
                      << "\",\"rss_bytes\":" << c.rss << ",\"memory_bytes\":" << c.physical
                      << ",\"read_bytes\":" << c.read << ",\"written_bytes\":" << c.written
                      << ",\"cpu_seconds\":" << c.cpu << '}';
        }
        std::cout << "]}" << std::endl;
        std::this_thread::sleep_for(std::chrono::milliseconds(interval));
    }
}
