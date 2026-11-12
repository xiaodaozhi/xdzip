#pragma once
#include "ImagePreviewProtocol.h"
#include <QImage>
#include <QString>
#include <functional>

namespace ImagePreview {
// The product passes only its fixed adjacent executable. Tests supply an owned
// fault worker to verify cancellation, framing and deadlines without parser races.
QImage render(const QString &worker, const QByteArray &bytes, int kind,
              const std::function<bool()> &current, int deadlineMs = DeadlineMs);
} // namespace ImagePreview
