// Composant vidéo : affiche libmpv dans la scène Qt Quick.
#pragma once

#include <QtCore/QStringList>
#include <QtCore/QUrl>
#include <QtQuick/QQuickFramebufferObject>

#include <mpv/client.h>
#include <mpv/render_gl.h>

class MpvRenderer;

class MpvItem : public QQuickFramebufferObject
{
    Q_OBJECT
    Q_PROPERTY(double position READ position NOTIFY positionChanged)
    Q_PROPERTY(double duration READ duration NOTIFY durationChanged)
    Q_PROPERTY(bool paused READ paused WRITE setPaused NOTIFY pausedChanged)
    Q_PROPERTY(double volume READ volume WRITE setVolume NOTIFY volumeChanged)
    Q_PROPERTY(double speed READ speed WRITE setSpeed NOTIFY speedChanged)
    Q_PROPERTY(QString mediaTitle READ mediaTitle NOTIFY mediaTitleChanged)
    Q_PROPERTY(bool hasMedia READ hasMedia NOTIFY hasMediaChanged)

public:
    explicit MpvItem(QQuickItem *parent = nullptr);
    ~MpvItem() override;

    Renderer *createRenderer() const override;

    double position() const { return m_position; }
    double duration() const { return m_duration; }
    bool paused() const { return m_paused; }
    double volume() const { return m_volume; }
    double speed() const { return m_speed; }
    QString mediaTitle() const { return m_mediaTitle; }
    bool hasMedia() const { return m_hasMedia; }

    void setPaused(bool paused);
    void setVolume(double volume);
    void setSpeed(double speed);

    Q_INVOKABLE void loadFile(const QUrl &url);
    Q_INVOKABLE void togglePause();
    Q_INVOKABLE void seekAbsolute(double seconds);
    Q_INVOKABLE void seekRelative(double seconds);
    Q_INVOKABLE void frameStep(bool forward);
    Q_INVOKABLE void screenshot();
    // Accès générique à toute commande mpv : command(["cycle", "sub"])
    Q_INVOKABLE void command(const QStringList &args);

signals:
    void positionChanged();
    void durationChanged();
    void pausedChanged();
    void volumeChanged();
    void speedChanged();
    void mediaTitleChanged();
    void hasMediaChanged();
    void fileLoaded();
    void endOfFile();

private slots:
    void handleEvents();

private:
    void handlePropertyChange(const mpv_event_property *prop);
    void setHasMedia(bool value);

    friend class MpvRenderer;
    mpv_handle *m_mpv = nullptr;
    mpv_render_context *m_renderCtx = nullptr;

    double m_position = 0.0;
    double m_duration = 0.0;
    bool m_paused = false;
    double m_volume = 100.0;
    double m_speed = 1.0;
    QString m_mediaTitle;
    bool m_hasMedia = false;
};

// Appelé depuis Rust avant la création de la fenêtre.
void lumen_init_video();
