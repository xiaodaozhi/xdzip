// One bounded image per process. Archive paths, passwords and target handles never enter it.
#include "ImagePreviewProtocol.h"
#include <QBuffer>
#include <QColorSpace>
#include <QGuiApplication>
#include <QImageReader>
#include <QJsonArray>
#include <QJsonDocument>
#include <QPainter>
#include <QRegularExpression>
#include <QSet>
#include <QSvgRenderer>
#include <QXmlStreamReader>
#include <QXmlStreamWriter>
#include <cstdio>
#include <limits>
#ifdef Q_OS_WIN
#define NOMINMAX
#include <fcntl.h>
#include <io.h>
#include <windows.h>
#else
#include <sys/resource.h>
#ifdef Q_OS_MACOS
#include <mach/mach.h>
#endif
#endif

namespace {
// Apply limits before accepting image bytes. Failure refuses decoding. On macOS,
// the address-space allowance excludes dyld's existing virtual reservation; it is
// not an RSS claim. The parent separately enforces a wall deadline and cancellation.
bool limits() {
    constexpr quint64 memory = 512 * 1024 * 1024;
#ifdef Q_OS_WIN
    HANDLE job = CreateJobObjectW(nullptr, nullptr);
    if (!job)
        return false;
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limit{};
    limit.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY |
                                             JOB_OBJECT_LIMIT_PROCESS_TIME |
                                             JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
    limit.BasicLimitInformation.PerProcessUserTimeLimit.QuadPart = 3 * 10000000LL;
    limit.ProcessMemoryLimit = SIZE_T(memory);
    if (!SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limit, sizeof(limit)) ||
        !AssignProcessToJobObject(job, GetCurrentProcess())) {
        CloseHandle(job);
        return false;
    }
    // The process owns this non-inherited job handle until OS teardown.
    return true;
#else
    quint64 addressSpace = memory;
#ifdef Q_OS_MACOS
    mach_task_basic_info_data_t info{};
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    if (task_info(mach_task_self(), MACH_TASK_BASIC_INFO, reinterpret_cast<task_info_t>(&info),
                  &count) != KERN_SUCCESS ||
        info.virtual_size > std::numeric_limits<quint64>::max() - memory)
        return false;
    addressSpace += info.virtual_size;
#endif
    const rlimit space{rlim_t(addressSpace), rlim_t(addressSpace)};
    const rlimit cpu{3, 3}, core{0, 0}, files{0, 0}, descriptors{64, 64};
    return setrlimit(RLIMIT_AS, &space) == 0 && setrlimit(RLIMIT_CPU, &cpu) == 0 &&
           setrlimit(RLIMIT_CORE, &core) == 0 && setrlimit(RLIMIT_FSIZE, &files) == 0 &&
           setrlimit(RLIMIT_NOFILE, &descriptors) == 0;
#endif
}
bool dimensions(QSize size) {
    return size.isValid() && size.width() <= ImagePreview::SourceDimension &&
           size.height() <= ImagePreview::SourceDimension &&
           qint64(size.width()) * size.height() <= ImagePreview::SourcePixels;
}
bool css(const QString &value) {
    if (value.contains('\\') || value.contains('@'))
        return false;
    static const QRegularExpression url(QStringLiteral("url\\s*\\("),
                                        QRegularExpression::CaseInsensitiveOption);
    static const QRegularExpression fragment(
        QStringLiteral("^\\s*(['\"]?)#[A-Za-z_][A-Za-z0-9_.:-]*\\1\\s*$"));
    auto matches = url.globalMatch(value);
    while (matches.hasNext()) {
        const auto match = matches.next();
        const auto end = value.indexOf(')', match.capturedEnd());
        if (end < 0 ||
            !fragment.match(value.mid(match.capturedEnd(), end - match.capturedEnd())).hasMatch())
            return false;
    }
    return true;
}
QByteArray svg(const QByteArray &input) {
    // Re-emit only the static SVG vocabulary before Qt sees it. Image elements,
    // foreign content, processing instructions, entities and resource paths are
    // rejected, rather than relying on the renderer to ignore them.
    static const QSet<QString> elements = {
        "svg",         "g",        "defs",           "symbol",
        "use",         "path",     "rect",           "circle",
        "ellipse",     "line",     "polyline",       "polygon",
        "text",        "tspan",    "textPath",       "title",
        "desc",        "style",    "linearGradient", "radialGradient",
        "stop",        "clipPath", "mask",           "pattern",
        "marker",      "filter",   "feBlend",        "feColorMatrix",
        "feComposite", "feFlood",  "feGaussianBlur", "feMerge",
        "feMergeNode", "feOffset"};
    QXmlStreamReader reader(input);
    reader.setEntityExpansionLimit(0);
    QByteArray output;
    QXmlStreamWriter writer(&output);
    int depth = 0, count = 0, ignoredMetadata = 0, styleDepth = 0;
    QString styleText;
    bool root = false;
    while (!reader.atEnd()) {
        const auto token = reader.readNext();
        if (token == QXmlStreamReader::DTD || token == QXmlStreamReader::EntityReference ||
            token == QXmlStreamReader::ProcessingInstruction)
            return {};
        if (token == QXmlStreamReader::StartElement) {
            if (++depth > 64 || ++count > 8192 || reader.attributes().size() > 64 || styleDepth)
                return {};
            if (ignoredMetadata)
                continue;
            const auto name = reader.name().toString();
            const auto ns = reader.namespaceUri();
            if (!root) {
                if (name != "svg")
                    return {};
                root = true;
            }
            if (name == "metadata") {
                ignoredMetadata = depth;
                continue;
            }
            if ((!ns.isEmpty() && ns != u"http://www.w3.org/2000/svg") || !elements.contains(name))
                return {};
            writer.writeStartElement(name);
            if (name == "style")
                styleDepth = depth;
            if (depth == 1) {
                writer.writeDefaultNamespace("http://www.w3.org/2000/svg");
                writer.writeNamespace("http://www.w3.org/1999/xlink", "xlink");
            }
            for (const auto &attribute : reader.attributes()) {
                const auto key = attribute.name().toString();
                const auto value = attribute.value().toString();
                const auto attrNs = attribute.namespaceUri();
                if (key.startsWith("on", Qt::CaseInsensitive) || value.size() > 1024 * 1024 ||
                    (!attrNs.isEmpty() && attrNs != u"http://www.w3.org/1999/xlink" &&
                     attrNs != u"http://www.w3.org/XML/1998/namespace") ||
                    !css(value))
                    return {};
                if (key == "href") {
                    static const QRegularExpression local(
                        QStringLiteral("^#[A-Za-z_][A-Za-z0-9_.:-]*$"));
                    if (!local.match(value).hasMatch())
                        return {};
                    writer.writeAttribute("xlink:href", value);
                } else if (key == "base" || key == "src") {
                    return {};
                } else if (attrNs == u"http://www.w3.org/XML/1998/namespace") {
                    writer.writeAttribute("xml:" + key, value);
                } else {
                    writer.writeAttribute(key, value);
                }
            }
        } else if (token == QXmlStreamReader::EndElement) {
            if (styleDepth == depth) {
                // XML comments and CDATA may divide one CSS token. Validate the
                // complete text that the renderer receives, never each chunk.
                if (!css(styleText))
                    return {};
                writer.writeCharacters(styleText);
                styleText.clear();
                styleDepth = 0;
            }
            if (ignoredMetadata) {
                if (ignoredMetadata == depth)
                    ignoredMetadata = 0;
            } else {
                writer.writeEndElement();
            }
            --depth;
        } else if (token == QXmlStreamReader::Characters && !ignoredMetadata) {
            const auto text = reader.text().toString();
            if (styleDepth) {
                styleText += text;
                if (styleText.size() > ImagePreview::EncodedLimit)
                    return {};
            } else {
                writer.writeCharacters(text);
            }
        }
        if (output.size() > ImagePreview::EncodedLimit)
            return {};
    }
    return reader.hasError() || !root || depth != 0 || writer.hasError() ? QByteArray{} : output;
}
QImage decode(QByteArray &input, int kind) {
    if (kind == 11) {
        auto safe = svg(input);
        if (safe.isEmpty())
            return {};
        QSvgRenderer renderer;
        renderer.setAnimationEnabled(false);
        const bool valid = renderer.load(safe);
        safe.fill('\0');
        if (!valid || !dimensions(renderer.defaultSize()))
            return {};
        const auto size = renderer.defaultSize().scaled(QSize(512, 512), Qt::KeepAspectRatio);
        QImage image(size, QImage::Format_RGBA8888_Premultiplied);
        if (image.isNull())
            return {};
        image.fill(Qt::transparent);
        QPainter painter(&image);
        renderer.render(&painter);
        return image;
    }
    if (kind == 9) {
        // Qt's TGA reader accepts uncompressed true-color TGA 2.0. Validate the
        // complete pixel span ourselves and supply the empty 2.0 footer for the
        // same legacy layout; no guessed or truncated pixels reach the decoder.
        if (input.size() < 18 || input[0] != 0 || input[1] != 0 || input[2] != 2 ||
            (uchar(input[16]) != 16 && uchar(input[16]) != 24 && uchar(input[16]) != 32) ||
            (uchar(input[17]) & 0x10) != 0)
            return {};
        const QSize size(qFromLittleEndian<quint16>(input.constData() + 12),
                         qFromLittleEndian<quint16>(input.constData() + 14));
        if (!dimensions(size))
            return {};
        const qint64 pixels = qint64(size.width()) * size.height() * (uchar(input[16]) / 8);
        if (input.size() < 18 + pixels)
            return {};
        const QByteArray signature("TRUEVISION-XFILE.\0", 18);
        if (input.size() == 18 + pixels) {
            if (input.size() > ImagePreview::EncodedLimit - 26)
                return {};
            input.append(8, '\0');
            input.append(signature);
        } else if (input.size() < 18 + pixels + 26 || !input.endsWith(signature)) {
            return {};
        }
    }
    if (kind == 10) {
        if (input.size() < 8 || input.first(4) != "icns" ||
            qFromBigEndian<quint32>(input.constData() + 4) != quint32(input.size()))
            return {};
        qsizetype position = 8;
        int chunks = 0;
        while (position < input.size()) {
            if (++chunks > 256 || input.size() - position < 8)
                return {};
            const auto length = qFromBigEndian<quint32>(input.constData() + position + 4);
            if (length < 8 || length > quint32(input.size() - position))
                return {};
            const auto type = input.mid(position, 4);
            const auto payload = input.mid(position + 8, length - 8);
            if (type != "TOC " && type != "icnV") {
                // ICNS otherwise permits generic nested format autodetection.
                // Admit only signatures for which its plugin forces PNG/JP2.
                if (payload.startsWith(QByteArray("\x89PNG\r\n\x1a\n\0\0\0\r", 12))) {
                    if (payload.size() < 33 ||
                        !dimensions(QSize(int(qFromBigEndian<quint32>(payload.constData() + 16)),
                                          int(qFromBigEndian<quint32>(payload.constData() + 20)))))
                        return {};
                } else if (!payload.startsWith(QByteArray("\0\0\0\x0cjP  \r\n\x87\n", 12))) {
                    return {};
                }
            }
            position += length;
        }
    }
    QBuffer buffer(&input);
    if (!buffer.open(QIODevice::ReadOnly))
        return {};
    const auto format = ImagePreview::format(kind);
    if (format.isEmpty())
        return {};
    QImageReader reader(&buffer, format);
    reader.setAutoDetectImageFormat(false);
    reader.setDecideFormatFromContent(false);
    reader.setAutoTransform(true);
    const auto original = reader.size();
    // The ICNS plugin does not implement Size. Its nested image allocation is
    // bounded by QImageReader and the worker; validate actual pixels after read.
    if (!dimensions(original) && kind != 10)
        return {};
    if (original.isValid())
        reader.setScaledSize(original.scaled(QSize(512, 512), Qt::KeepAspectRatio));
    auto image = reader.read();
    if (image.isNull() || reader.error() != QImageReader::UnknownError || !dimensions(image.size()))
        return {};
    if (kind == 9 && (uchar(input[17]) & 0x0f) == 0)
        image = image.convertToFormat(QImage::Format_RGB32);
    const auto thumbnail = image.size().scaled(QSize(512, 512), Qt::KeepAspectRatio);
    if (image.size() != thumbnail)
        image = image.scaled(thumbnail, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    const QColorSpace srgb(QColorSpace::SRgb);
    if (image.colorSpace().isValid() && image.colorSpace() != srgb)
        image = image.convertedToColorSpace(srgb);
    return image;
}
} // namespace
int main(int argc, char **argv) {
#ifdef Q_OS_WIN
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
#endif
    QGuiApplication application(argc, argv);
    if (!limits())
        return 2;
    QImageReader::setAllocationLimit(128);
    if (argc == 2 && QByteArray(argv[1]) == "--formats") {
        const auto supported = QImageReader::supportedImageFormats();
        QJsonArray formats;
        for (int kind = 2; kind <= 12; ++kind) {
            const auto name = ImagePreview::format(kind);
            if (!name.isEmpty() && (kind == 11 || supported.contains(name)))
                formats.append(QString::fromLatin1(name));
        }
        const auto report = QJsonDocument(formats).toJson(QJsonDocument::Compact);
        return fwrite(report.data(), 1, size_t(report.size()), stdout) == size_t(report.size()) ? 0
                                                                                                : 2;
    }
    if (argc != 1)
        return 2;
    QByteArray header(12, '\0');
    if (fread(header.data(), 1, 12, stdin) != 12 || header.first(4) != "XIP1")
        return 2;
    const auto kind = ImagePreview::read32(header, 4);
    const auto length = ImagePreview::read32(header, 8);
    if (kind > 12 || ImagePreview::format(int(kind)).isEmpty() || length == 0 ||
        length > ImagePreview::EncodedLimit)
        return 2;
    QByteArray input(qsizetype(length), '\0');
    if (fread(input.data(), 1, length, stdin) != length || fgetc(stdin) != EOF)
        return 2;
    QImage image;
    try {
        image = decode(input, int(kind)).convertToFormat(QImage::Format_RGBA8888);
    } catch (...) {
        input.fill('\0');
        return 2;
    }
    input.fill('\0');
    if (image.isNull() || image.width() > 512 || image.height() > 512 ||
        image.sizeInBytes() > ImagePreview::OutputLimit ||
        image.bytesPerLine() != image.width() * 4)
        return 2;
    QByteArray output("XIR1", 4);
    ImagePreview::append32(output, quint32(image.width()));
    ImagePreview::append32(output, quint32(image.height()));
    ImagePreview::append32(output, quint32(image.sizeInBytes()));
    if (fwrite(output.data(), 1, size_t(output.size()), stdout) != size_t(output.size()) ||
        fwrite(image.constBits(), 1, size_t(image.sizeInBytes()), stdout) !=
            size_t(image.sizeInBytes()) ||
        fflush(stdout) != 0)
        return 2;
    return 0;
}
