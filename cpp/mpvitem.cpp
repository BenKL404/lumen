#include "mpvitem.h"

#include <clocale>
#include <cstring>
#include <vector>

#include <QtGui/QGuiApplication>
#include <QtGui/QOpenGLContext>
#include <QtGui/qguiapplication_platform.h>
#include <QtOpenGL/QOpenGLFramebufferObject>
#include <QtQml/qqml.h>
#include <QtQuick/QQuickOpenGLUtils>
#include <QtQuick/QQuickWindow>

namespace {

void *getProcAddress(void *, const char *name)
{
    QOpenGLContext *ctx = QOpenGLContext::currentContext();
    return ctx ? reinterpret_cast<void *>(ctx->getProcAddress(QByteArray(name))) : nullptr;
}

// Convertit un nœud mpv (listes, dictionnaires, scalaires) en valeur lisible par QML
QVariant nodeToVariant(const mpv_node *node)
{
    switch (node->format) {
    case MPV_FORMAT_STRING:
        return QString::fromUtf8(node->u.string);
    case MPV_FORMAT_FLAG:
        return node->u.flag != 0;
    case MPV_FORMAT_INT64:
        return static_cast<qlonglong>(node->u.int64);
    case MPV_FORMAT_DOUBLE:
        return node->u.double_;
    case MPV_FORMAT_NODE_ARRAY: {
        QVariantList list;
        for (int i = 0; i < node->u.list->num; ++i)
            list.append(nodeToVariant(&node->u.list->values[i]));
        return list;
    }
    case MPV_FORMAT_NODE_MAP: {
        QVariantMap map;
        for (int i = 0; i < node->u.list->num; ++i)
            map.insert(QString::fromUtf8(node->u.list->keys[i]), nodeToVariant(&node->u.list->values[i]));
        return map;
    }
    default:
        return {};
    }
}

// mpv signale qu'une nouvelle image est prête -> redessiner (thread GUI)
void onMpvRedraw(void *ctx)
{
    QMetaObject::invokeMethod(static_cast<MpvItem *>(ctx), "update", Qt::QueuedConnection);
}

// mpv a des événements en attente -> les traiter dans le thread GUI
void onMpvWakeup(void *ctx)
{
    QMetaObject::invokeMethod(static_cast<MpvItem *>(ctx), "handleEvents", Qt::QueuedConnection);
}

} // namespace

// ---------------------------------------------------------------------------
// Renderer : s'exécute dans le thread de rendu de Qt Quick
// ---------------------------------------------------------------------------
class MpvRenderer : public QQuickFramebufferObject::Renderer
{
public:
    explicit MpvRenderer(MpvItem *item) : m_item(item) {}

    QOpenGLFramebufferObject *createFramebufferObject(const QSize &size) override
    {
        if (!m_item->m_renderCtx) {
            mpv_opengl_init_params glInit{getProcAddress, nullptr};
            std::vector<mpv_render_param> params{
                {MPV_RENDER_PARAM_API_TYPE, const_cast<char *>(MPV_RENDER_API_TYPE_OPENGL)},
                {MPV_RENDER_PARAM_OPENGL_INIT_PARAMS, &glInit},
            };

            // Transmettre l'affichage natif permet le décodage matériel
            // VA-API sans copie (zero-copy) sous X11 et Wayland.
#if QT_CONFIG(xcb)
            if (auto *x11 = qApp->nativeInterface<QNativeInterface::QX11Application>())
                params.push_back({MPV_RENDER_PARAM_X11_DISPLAY, x11->display()});
#endif
#if QT_VERSION >= QT_VERSION_CHECK(6, 5, 0) && QT_CONFIG(wayland)
            if (auto *wl = qApp->nativeInterface<QNativeInterface::QWaylandApplication>())
                params.push_back({MPV_RENDER_PARAM_WL_DISPLAY, wl->display()});
#endif
            params.push_back({MPV_RENDER_PARAM_INVALID, nullptr});

            if (mpv_render_context_create(&m_item->m_renderCtx, m_item->m_mpv, params.data()) < 0)
                qFatal("Impossible d'initialiser le rendu mpv");

            mpv_render_context_set_update_callback(m_item->m_renderCtx, onMpvRedraw, m_item);
        }
        return QQuickFramebufferObject::Renderer::createFramebufferObject(size);
    }

    void render() override
    {
        QOpenGLFramebufferObject *fbo = framebufferObject();
        mpv_opengl_fbo mpvFbo{static_cast<int>(fbo->handle()), fbo->width(), fbo->height(), 0};
        int flipY = 0;
        mpv_render_param params[] = {
            {MPV_RENDER_PARAM_OPENGL_FBO, &mpvFbo},
            {MPV_RENDER_PARAM_FLIP_Y, &flipY},
            {MPV_RENDER_PARAM_INVALID, nullptr},
        };
        mpv_render_context_render(m_item->m_renderCtx, params);
        // mpv modifie l'état OpenGL : on le restaure pour Qt Quick
        QQuickOpenGLUtils::resetOpenGLState();
    }

private:
    MpvItem *m_item;
};

// ---------------------------------------------------------------------------
// MpvItem
// ---------------------------------------------------------------------------
MpvItem::MpvItem(QQuickItem *parent) : QQuickFramebufferObject(parent)
{
    // Exigence de libmpv : séparateur décimal "C"
    std::setlocale(LC_NUMERIC, "C");

    m_mpv = mpv_create();
    if (!m_mpv)
        qFatal("Impossible de créer l'instance mpv");

    mpv_set_option_string(m_mpv, "vo", "libmpv");
    mpv_set_option_string(m_mpv, "hwdec", "auto-safe");      // décodage matériel
    mpv_set_option_string(m_mpv, "keep-open", "yes");        // rester sur la dernière image
    mpv_set_option_string(m_mpv, "input-default-bindings", "no");
    mpv_set_option_string(m_mpv, "screenshot-directory", "~~desktop/");
    mpv_set_option_string(m_mpv, "terminal", "yes");
    mpv_set_option_string(m_mpv, "msg-level", "all=warn");

    if (mpv_initialize(m_mpv) < 0)
        qFatal("Impossible d'initialiser mpv");

    mpv_observe_property(m_mpv, 0, "time-pos", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "duration", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "pause", MPV_FORMAT_FLAG);
    mpv_observe_property(m_mpv, 0, "volume", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "speed", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "media-title", MPV_FORMAT_STRING);
    mpv_observe_property(m_mpv, 0, "track-list", MPV_FORMAT_NODE);
    mpv_observe_property(m_mpv, 0, "sub-delay", MPV_FORMAT_DOUBLE);
    mpv_observe_property(m_mpv, 0, "eof-reached", MPV_FORMAT_FLAG);

    mpv_set_wakeup_callback(m_mpv, onMpvWakeup, this);

    // Le FBO de Qt est inversé verticalement par rapport à mpv
    setMirrorVertically(true);
}

MpvItem::~MpvItem()
{
    if (m_renderCtx)
        mpv_render_context_free(m_renderCtx);
    mpv_terminate_destroy(m_mpv);
}

QQuickFramebufferObject::Renderer *MpvItem::createRenderer() const
{
    window()->setPersistentGraphics(true);
    window()->setPersistentSceneGraph(true);
    return new MpvRenderer(const_cast<MpvItem *>(this));
}

void MpvItem::handleEvents()
{
    while (m_mpv) {
        mpv_event *event = mpv_wait_event(m_mpv, 0);
        if (event->event_id == MPV_EVENT_NONE)
            break;

        switch (event->event_id) {
        case MPV_EVENT_PROPERTY_CHANGE:
            handlePropertyChange(static_cast<mpv_event_property *>(event->data));
            break;
        case MPV_EVENT_FILE_LOADED:
            setHasMedia(true);
            emit fileLoaded();
            break;
        case MPV_EVENT_END_FILE:
            emit endOfFile();
            break;
        default:
            break;
        }
    }
}

void MpvItem::handlePropertyChange(const mpv_event_property *prop)
{
    const bool available = prop->format != MPV_FORMAT_NONE && prop->data;
    const char *name = prop->name;

    if (std::strcmp(name, "time-pos") == 0) {
        m_position = available ? *static_cast<double *>(prop->data) : 0.0;
        emit positionChanged();
    } else if (std::strcmp(name, "duration") == 0) {
        m_duration = available ? *static_cast<double *>(prop->data) : 0.0;
        emit durationChanged();
    } else if (std::strcmp(name, "pause") == 0 && available) {
        m_paused = *static_cast<int *>(prop->data) != 0;
        emit pausedChanged();
    } else if (std::strcmp(name, "volume") == 0 && available) {
        m_volume = *static_cast<double *>(prop->data);
        emit volumeChanged();
    } else if (std::strcmp(name, "speed") == 0 && available) {
        m_speed = *static_cast<double *>(prop->data);
        emit speedChanged();
    } else if (std::strcmp(name, "media-title") == 0) {
        m_mediaTitle = available ? QString::fromUtf8(*static_cast<char **>(prop->data)) : QString();
        emit mediaTitleChanged();
    } else if (std::strcmp(name, "track-list") == 0) {
        m_tracks = available && prop->format == MPV_FORMAT_NODE
            ? nodeToVariant(static_cast<mpv_node *>(prop->data)).toList()
            : QVariantList();
        emit tracksChanged();
    } else if (std::strcmp(name, "sub-delay") == 0 && available) {
        m_subDelay = *static_cast<double *>(prop->data);
        emit subDelayChanged();
    } else if (std::strcmp(name, "eof-reached") == 0) {
        const bool reached = available && *static_cast<int *>(prop->data) != 0;
        if (reached != m_eofReached) {
            m_eofReached = reached;
            emit eofReachedChanged();
        }
    }
}

void MpvItem::setHasMedia(bool value)
{
    if (m_hasMedia != value) {
        m_hasMedia = value;
        emit hasMediaChanged();
    }
}

void MpvItem::setPaused(bool paused)
{
    int flag = paused ? 1 : 0;
    mpv_set_property_async(m_mpv, 0, "pause", MPV_FORMAT_FLAG, &flag);
}

void MpvItem::setVolume(double volume)
{
    double v = qBound(0.0, volume, 130.0);
    mpv_set_property_async(m_mpv, 0, "volume", MPV_FORMAT_DOUBLE, &v);
}

void MpvItem::setSpeed(double speed)
{
    double s = qBound(0.25, speed, 4.0);
    mpv_set_property_async(m_mpv, 0, "speed", MPV_FORMAT_DOUBLE, &s);
}

void MpvItem::setSubDelay(double seconds)
{
    mpv_set_property_async(m_mpv, 0, "sub-delay", MPV_FORMAT_DOUBLE, &seconds);
}

void MpvItem::loadFile(const QUrl &url)
{
    const QString target = url.isLocalFile() ? url.toLocalFile() : url.toString();
    command({QStringLiteral("loadfile"), target});
    setPaused(false);
}

void MpvItem::togglePause()
{
    setPaused(!m_paused);
}

void MpvItem::seekAbsolute(double seconds)
{
    command({QStringLiteral("seek"), QString::number(seconds), QStringLiteral("absolute")});
}

void MpvItem::seekRelative(double seconds)
{
    command({QStringLiteral("seek"), QString::number(seconds), QStringLiteral("relative")});
}

void MpvItem::frameStep(bool forward)
{
    command({forward ? QStringLiteral("frame-step") : QStringLiteral("frame-back-step")});
}

void MpvItem::screenshot()
{
    command({QStringLiteral("screenshot")});
}

void MpvItem::command(const QStringList &args)
{
    std::vector<QByteArray> storage;
    storage.reserve(args.size());
    std::vector<const char *> argv;
    argv.reserve(args.size() + 1);
    for (const QString &arg : args) {
        storage.push_back(arg.toUtf8());
        argv.push_back(storage.back().constData());
    }
    argv.push_back(nullptr);
    mpv_command_async(m_mpv, 0, argv.data());
}

void lumen_init_video()
{
    QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL);
    qmlRegisterType<MpvItem>("com.lumen.video", 1, 0, "MpvVideo");
}
