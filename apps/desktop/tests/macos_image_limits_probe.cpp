// Standalone diagnostic only. It compiles the real worker with an OS-call
// interceptor to omit one limit at a time; product builds never include it.
#include "ImagePreviewProcess.h"
#include <QFile>
#include <QJsonObject>
#include <cerrno>
#include <csignal>
#include <cstring>
#include <sys/resource.h>
#include <unistd.h>

namespace {
const char *omittedLimit = "";
int imageProbeSetLimit(int resource, const rlimit *limit) {
    if (resource == RLIMIT_FSIZE) {
        rlim_t replacement = 0;
        if (std::strcmp(omittedLimit, "fsize64k") == 0)
            replacement = 64 * 1024;
        else if (std::strcmp(omittedLimit, "fsize1m") == 0)
            replacement = 1024 * 1024;
        else if (std::strcmp(omittedLimit, "fsize16m") == 0)
            replacement = 16 * 1024 * 1024;
        else if (std::strcmp(omittedLimit, "fsize64m") == 0)
            replacement = 64 * 1024 * 1024;
        else if (std::strcmp(omittedLimit, "fsize128m") == 0)
            replacement = 128 * 1024 * 1024;
        else if (std::strcmp(omittedLimit, "fsize512m") == 0)
            replacement = 512 * 1024 * 1024;
        if (replacement) {
            const rlimit bounded{replacement, replacement};
            return ::setrlimit(resource, &bounded);
        }
    }
    if (std::strcmp(omittedLimit, "all") == 0 ||
        (resource == RLIMIT_AS && std::strcmp(omittedLimit, "as") == 0) ||
        (resource == RLIMIT_FSIZE && std::strcmp(omittedLimit, "fsize") == 0) ||
        (resource == RLIMIT_NOFILE && std::strcmp(omittedLimit, "nofile") == 0) ||
        (resource == RLIMIT_CPU && std::strcmp(omittedLimit, "cpu") == 0))
        return 0;
    return ::setrlimit(resource, limit);
}
} // namespace

#define setrlimit imageProbeSetLimit
#define main imagePreviewWorkerEntryForProbe
#include "../../../tools/image-preview/main.cpp"
#undef main
#undef setrlimit

int main(int argc, char **argv) {
    if (argc == 2 && std::strncmp(argv[1], "worker-", 7) == 0) {
        omittedLimit = argv[1] + 7;
        char *workerArguments[] = {argv[0], nullptr};
        return imagePreviewWorkerEntryForProbe(1, workerArguments);
    }
    QGuiApplication application(argc, argv);
    QImageReader::setAllocationLimit(128);
    QJsonArray records;
    if (argc == 2 && std::strcmp(argv[1], "file-boundary") == 0) {
        if (!limits())
            return 2;
        std::signal(SIGXFSZ, SIG_IGN);
        FILE *file = tmpfile();
        if (!file)
            return 2;
        const bool allowed = ftruncate(fileno(file), ImagePreview::RuntimeFileLimit) == 0;
        errno = 0;
        const bool refused =
            ftruncate(fileno(file), ImagePreview::RuntimeFileLimit + 1) != 0 && errno == EFBIG;
        fclose(file);
        const auto record = QJsonDocument(QJsonObject{{"limit", ImagePreview::RuntimeFileLimit},
                                                      {"at_limit_allowed", allowed},
                                                      {"over_limit_refused", refused}})
                                .toJson(QJsonDocument::Compact);
        fwrite(record.data(), 1, size_t(record.size()), stdout);
        return allowed && refused ? 0 : 2;
    }
    if (argc >= 4 && std::strcmp(argv[1], "parent") == 0) {
        for (int i = 3; i < argc; ++i) {
            QFile file(QString::fromUtf8(argv[i]));
            if (!file.open(QIODevice::ReadOnly))
                return 2;
            const auto image = ImagePreview::render(QString::fromUtf8(argv[2]), file.readAll(), 12,
                                                    [] { return true; });
            const auto pixel = image.isNull() ? QColor() : image.pixelColor(image.rect().center());
            records.append(QJsonObject{
                {"decoded", !image.isNull()},
                {"width", image.width()},
                {"height", image.height()},
                {"pixel", QJsonArray{pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()}}});
        }
        const auto result = QJsonDocument(records).toJson(QJsonDocument::Compact);
        return fwrite(result.data(), 1, size_t(result.size()), stdout) == size_t(result.size()) ? 0
                                                                                                : 2;
    }
    for (int i = 1; i < argc; ++i) {
        QImageReader reader(QString::fromUtf8(argv[i]), "heic");
        reader.setAutoDetectImageFormat(false);
        reader.setDecideFormatFromContent(false);
        reader.setAutoTransform(true);
        const auto size = reader.size();
        if (size.isValid())
            reader.setScaledSize(size.scaled(QSize(512, 512), Qt::KeepAspectRatio));
        const auto image = reader.read();
        mach_task_basic_info_data_t info{};
        mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
        task_info(mach_task_self(), MACH_TASK_BASIC_INFO, reinterpret_cast<task_info_t>(&info),
                  &count);
        records.append(QJsonObject{{"reader_error", int(reader.error())},
                                   {"source_width", size.width()},
                                   {"source_height", size.height()},
                                   {"width", image.width()},
                                   {"height", image.height()},
                                   {"decoded", !image.isNull()},
                                   {"virtual_bytes", double(info.virtual_size)},
                                   {"resident_bytes", double(info.resident_size)}});
    }
    const auto result = QJsonDocument(records).toJson(QJsonDocument::Compact);
    return fwrite(result.data(), 1, size_t(result.size()), stdout) == size_t(result.size()) ? 0 : 2;
}
