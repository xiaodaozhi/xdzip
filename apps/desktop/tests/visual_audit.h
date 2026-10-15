#pragma once
#include <QDir>
#include <QFile>
#include <QFont>
#include <QFontInfo>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QQmlComponent>
#include <QQmlProperty>
#include <QQuickItem>
#include <QQuickWindow>
#include <QScreen>
#include <QSysInfo>
#include <QTest>
#include <memory>

// Opt-in evidence capture, not a golden-image approval. Geometry uses Qt logical pixels;
// screenshots are untouched grabWindow output and may have a different physical DPR.
namespace visual_audit {
inline void collect(QQuickItem *item, QJsonArray &nodes, const QString &path = "0") {
    // TableView can cull pooled delegates without changing QQuickItem::isVisible().
    // They are not painted, so their old model identities cannot be measured as UI.
    if (item->property("inReusePool").toBool())
        return;
    const auto position = item->mapToScene(QPointF());
    QJsonObject node{{"path", path},
                     {"class", item->metaObject()->className()},
                     {"name", item->objectName()},
                     {"x", position.x()},
                     {"y", position.y()},
                     {"width", item->width()},
                     {"height", item->height()},
                     {"visible", item->isVisible()},
                     {"clip", item->clip()},
                     {"baselineOffset", item->baselineOffset()},
                     {"implicitWidth", item->implicitWidth()},
                     {"implicitHeight", item->implicitHeight()}};
    for (const auto *key :
         {"text", "color", "source", "radius", "horizontalAlignment", "verticalAlignment",
          "contentWidth", "contentHeight", "opacity", "leftPadding", "rightPadding", "nodeDepth",
          "depth", "row", "expanded"}) {
        const auto value = item->property(key);
        if (value.isValid())
            node[key] = QJsonValue::fromVariant(value);
    }
    const auto fontValue = item->property("font");
    if (fontValue.isValid()) {
        const auto font = fontValue.value<QFont>();
        node["fontPixelSize"] = font.pixelSize();
        node["fontPointSize"] = font.pointSizeF();
        node["fontFamily"] = QFontInfo(font).family();
        node["resolvedPixelSize"] = QFontInfo(font).pixelSize();
        node["fontWeight"] = static_cast<int>(font.weight());
    }
    for (const auto *key : {"border.color", "border.width"}) {
        const auto value = QQmlProperty::read(item, QString::fromLatin1(key));
        if (value.isValid())
            node[QString::fromLatin1(key)] = QJsonValue::fromVariant(value);
    }
    nodes.append(node);
    const auto children = item->childItems();
    for (int i = 0; i < children.size(); ++i)
        collect(children[i], nodes, path + "/" + QString::number(i));
}
inline bool save(QQuickWindow *window, const QString &directory, const QString &name) {
    QTest::qWait(80); // allow layout, font fallback and the software render loop to settle
    const auto shot = window->grabWindow();
    QJsonArray nodes;
    collect(window->contentItem(), nodes);
    QJsonObject data{{"width", window->width()},
                     {"height", window->height()},
                     {"availableScreenHeight", window->screen()->availableGeometry().height()},
                     {"minimumWidth", window->minimumWidth()},
                     {"minimumHeight", window->minimumHeight()},
                     {"dpr", window->devicePixelRatio()},
                     {"imageWidth", shot.width()},
                     {"imageHeight", shot.height()},
                     {"qt", qVersion()},
                     {"platform", QGuiApplication::platformName()},
                     {"os", QSysInfo::prettyProductName()},
                     {"architecture", QSysInfo::currentCpuArchitecture()},
                     {"requestedQuickBackend", qEnvironmentVariable("QT_QUICK_BACKEND")},
                     {"nodes", nodes}};
    QFile file(directory + "/" + name + ".json");
    return file.open(QIODevice::WriteOnly) && file.write(QJsonDocument(data).toJson()) > 0 &&
           shot.save(directory + "/" + name + ".png");
}
inline bool capture(QQmlEngine &engine, QQuickWindow *browser) {
    const auto directory = qEnvironmentVariable("XDZIP_VISUAL_AUDIT");
    if (directory.isEmpty())
        return true;
    if (!QDir().mkpath(directory))
        return false;
    QQmlComponent settings(&engine);
    settings.setData(
        "import QtQuick\nimport \"qrc:/xdzip/ui\" as Ui\n"
        "QtObject { property real scale: 1; onScaleChanged: Ui.Theme.fontScale = scale }",
        QUrl());
    std::unique_ptr<QObject> scaling(settings.create());
    if (!scaling)
        return false;
    bool ok = save(browser, directory, "browser-light");
    browser->setProperty("darkMode", true);
    ok &= save(browser, directory, "browser-dark");
    browser->setProperty("darkMode", false);
    browser->resize(1024, 622);
    ok &= save(browser, directory, "browser-normal");
    browser->resize(browser->minimumWidth(), browser->minimumHeight());
    ok &= save(browser, directory, "browser-minimum");
    browser->resize(832, 544); // records whether the requested design minimum is attainable
    ok &= save(browser, directory, "browser-request-minimum");
    browser->resize(1088, 592);
    scaling->setProperty("scale", 2.0);
    ok &= save(browser, directory, "browser-large-type");
    browser->resize(832, 544);
    ok &= save(browser, directory, "browser-large-minimum");
    scaling->setProperty("scale", 1.0);

    QQmlComponent component(&engine, QUrl("qrc:/xdzip/ui/TaskWindow.qml"));
    std::unique_ptr<QObject> object(component.createWithInitialProperties(
        {{"owner", QVariant::fromValue(browser)},
         {"archiveName", QString::fromUtf8("北境旅行.zip")},
         {"destination", QString::fromUtf8("下载 / 北境旅行")},
         {"currentFile", QString::fromUtf8("旅途瞬间-08.jpg")},
         {"fraction", 0.6},
         {"totalKnown", true},
         {"stopping", false},
         {"processedSummary", QString::fromUtf8("已处理 46.5 MB / 77.5 MB")},
         {"fileCountSummary", QString::fromUtf8("18 / 30 个文件")},
         {"speedSummary", "6.2 MB/s"},
         {"remainingSummary", QString::fromUtf8("约 5 秒")},
         {"phaseSummary", QString::fromUtf8("写入文件")}}));
    auto *task = qobject_cast<QQuickWindow *>(object.get());
    if (!task)
        return false;
    task->show();
    ok &= save(task, directory, "task-light");
    browser->setProperty("darkMode", true);
    ok &= save(task, directory, "task-dark");
    browser->setProperty("darkMode", false);
    scaling->setProperty("scale", 2.0);
    ok &= save(task, directory, "task-large-type");
    auto *scroll = task->findChild<QObject *>("taskScroll");
    auto *flickable = scroll ? scroll->property("contentItem").value<QObject *>() : nullptr;
    if (flickable) {
        flickable->setProperty("contentY", flickable->property("contentHeight").toDouble() -
                                               flickable->property("height").toDouble());
        ok &= save(task, directory, "task-large-type-bottom");
    } else {
        ok = false;
    }
    scaling->setProperty("scale", 1.0);
    task->close();
    return ok;
}
} // namespace visual_audit
