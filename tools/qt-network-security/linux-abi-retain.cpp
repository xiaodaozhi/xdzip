// SPDX-License-Identifier: Apache-2.0
// Keep the original SDK's exported builtin metatype constants across compilers.
#if defined(__aarch64__)
#include <arm_acle.h>
#endif
#include <QtCore/QtCore>

QT_BEGIN_NAMESPACE
namespace QtPrivate {
#define XDZIP_META_ABI_TYPES(APPLY)                                                                \
    APPLY(QByteArray)                                                                              \
    APPLY(QCborArray)                                                                              \
    APPLY(QCborValue)                                                                              \
    APPLY(QJsonArray)                                                                              \
    APPLY(QJsonValue)                                                                              \
    APPLY(QJsonObject)                                                                             \
    APPLY(QModelIndex)                                                                             \
    APPLY(QEasingCurve)                                                                            \
    APPLY(QJsonDocument)                                                                           \
    APPLY(QCborSimpleType)                                                                         \
    APPLY(QRegularExpression)                                                                      \
    APPLY(QPersistentModelIndex)                                                                   \
    APPLY(QMap<QString, QVariant>)                                                                 \
    APPLY(QUrl)                                                                                    \
    APPLY(QChar)                                                                                   \
    APPLY(QDate)                                                                                   \
    APPLY(QHash<QString, QVariant>)                                                                \
    APPLY(QLine)                                                                                   \
    APPLY(QList<QByteArray>)                                                                       \
    APPLY(QList<QString>)                                                                          \
    APPLY(QList<QVariant>)                                                                         \
    APPLY(QRect)                                                                                   \
    APPLY(QSize)                                                                                   \
    APPLY(QTime)                                                                                   \
    APPLY(QUuid)                                                                                   \
    APPLY(QLineF)                                                                                  \
    APPLY(QPoint)                                                                                  \
    APPLY(QRectF)                                                                                  \
    APPLY(QSizeF)                                                                                  \
    APPLY(QLocale)                                                                                 \
    APPLY(QPointF)                                                                                 \
    APPLY(QString)                                                                                 \
    APPLY(QCborMap)                                                                                \
    APPLY(QVariant)                                                                                \
    APPLY(qfloat16)                                                                                \
    APPLY(QBitArray)                                                                               \
    APPLY(QDateTime)                                                                               \
    APPLY(char32_t)                                                                                \
    APPLY(std::nullptr_t)                                                                          \
    APPLY(char16_t)                                                                                \
    APPLY(QObject *)                                                                               \
    APPLY(void *)                                                                                  \
    APPLY(std::pair<QVariant, QVariant>)                                                           \
    APPLY(signed char)                                                                             \
    APPLY(bool)                                                                                    \
    APPLY(char)                                                                                    \
    APPLY(double)                                                                                  \
    APPLY(float)                                                                                   \
    APPLY(unsigned char)                                                                           \
    APPLY(int)                                                                                     \
    APPLY(unsigned int)                                                                            \
    APPLY(long)                                                                                    \
    APPLY(unsigned long)                                                                           \
    APPLY(short)                                                                                   \
    APPLY(unsigned short)                                                                          \
    APPLY(long long)                                                                               \
    APPLY(unsigned long long)

#define XDZIP_INSTANTIATE(...)                                                                     \
    static_assert(QMetaTypeInterfaceWrapper<__VA_ARGS__>::IsConstMetaTypeInterface);               \
    template __attribute__((visibility("default")))                                                \
    const bool QMetaTypeInterfaceWrapper<__VA_ARGS__>::IsConstMetaTypeInterface;
XDZIP_META_ABI_TYPES(XDZIP_INSTANTIATE)
#undef XDZIP_INSTANTIATE

// Taking each address requires storage even when constant folding would omit it.
#define XDZIP_ADDRESS(...) &QMetaTypeInterfaceWrapper<__VA_ARGS__>::IsConstMetaTypeInterface,
__attribute__((used)) static const bool *const retainedMetaConstants[] = {
    XDZIP_META_ABI_TYPES(XDZIP_ADDRESS)};
#undef XDZIP_ADDRESS
#undef XDZIP_META_ABI_TYPES
} // namespace QtPrivate
QT_END_NAMESPACE
