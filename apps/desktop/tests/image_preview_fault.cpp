#include "ImagePreviewProtocol.h"
#include <chrono>
#include <cstdio>
#include <thread>
#ifdef Q_OS_WIN
#include <fcntl.h>
#include <io.h>
#endif
int main() {
#ifdef Q_OS_WIN
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
#endif
    QByteArray header(12, '\0');
    if (fread(header.data(), 1, 12, stdin) != 12)
        return 2;
    const auto kind = ImagePreview::read32(header, 4);
    if (kind == 2) {
        std::this_thread::sleep_for(std::chrono::seconds(30));
        return 0;
    }
    QByteArray output("XIR1", 4);
    ImagePreview::append32(output, 1);
    ImagePreview::append32(output, 1);
    ImagePreview::append32(output, 4);
    output.append(4, '\0');
    if (kind == 4)
        output.append('x');
    if (kind == 6)
        output.append(ImagePreview::OutputLimit + 32, 'x');
    if (fwrite(output.data(), 1, size_t(output.size()), stdout) != size_t(output.size()))
        return 2;
    return kind == 5 ? 1 : 0;
}
