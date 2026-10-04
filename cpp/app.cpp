#include "app.h"

#include <QtCore/QDir>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtQuick/QQuickWindow>

namespace {

QQuickWindow *rootWindow(const QQmlApplicationEngine &engine)
{
    const auto roots = engine.rootObjects();
    return roots.isEmpty() ? nullptr : qobject_cast<QQuickWindow *>(roots.first());
}

} // namespace

void lumen_init_app()
{
    QGuiApplication::setApplicationName(QStringLiteral("Lumen"));
    QGuiApplication::setDesktopFileName(QStringLiteral("lumen"));
}

void lumen_set_window_icon()
{
    QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/qt/qml/com/lumen/player/assets/lumen.svg")));
}

bool lumen_qml_loaded(const QQmlApplicationEngine &engine)
{
    return !engine.rootObjects().isEmpty();
}

void lumen_open_file(const QQmlApplicationEngine &engine, const QString &path)
{
    QQuickWindow *window = rootWindow(engine);
    if (!window)
        return;
    // Accepte un chemin (relatif ou absolu) comme une URL déjà formée
    const QUrl url = QUrl::fromUserInput(path, QDir::currentPath(), QUrl::AssumeLocalFile);
    QMetaObject::invokeMethod(window, "openUrl", Q_ARG(QVariant, QVariant(url)));
}

void lumen_snapshot(const QQmlApplicationEngine &engine, const QString &path, int delayMs)
{
    QQuickWindow *window = rootWindow(engine);
    if (!window)
        return;
    QTimer::singleShot(delayMs, window, [window, path] {
        if (!window->grabWindow().save(path))
            qWarning("Lumen : impossible d'enregistrer la capture %s", qPrintable(path));
        // Fermeture normale (et non quit()) : enregistre la position et les paramètres comme ✕
        window->close();
    });
}
