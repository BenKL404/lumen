// Composant vidéo : affiche libmpv dans la scène Qt Quick.
#pragma once

#include <QtCore/QStringList>
#include <QtCore/QUrl>
#include <QtCore/QVariantList>
#include <QtCore/QVariantMap>
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
    // Pistes du fichier (propriété mpv « track-list ») : id, type, title, lang, codec, selected…
    Q_PROPERTY(QVariantList tracks READ tracks NOTIFY tracksChanged)
    Q_PROPERTY(double subDelay READ subDelay WRITE setSubDelay NOTIFY subDelayChanged)
    // Vrai quand la lecture est arrivée au bout (keep-open : mpv reste sur la dernière image)
    Q_PROPERTY(bool eofReached READ eofReached NOTIFY eofReachedChanged)
    Q_PROPERTY(bool muted READ muted WRITE setMuted NOTIFY mutedChanged)
    // Infos techniques, indexées par nom de propriété mpv (video-format, width, audio-bitrate…)
    Q_PROPERTY(QVariantMap info READ info NOTIFY infoChanged)

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
    QVariantList tracks() const { return m_tracks; }
    double subDelay() const { return m_subDelay; }
    bool eofReached() const { return m_eofReached; }
    bool muted() const { return m_muted; }
    QVariantMap info() const { return m_info; }

    void setPaused(bool paused);
    void setVolume(double volume);
    void setSpeed(double speed);
    void setSubDelay(double seconds);
    void setMuted(bool muted);

    Q_INVOKABLE void loadFile(const QUrl &url);
    Q_INVOKABLE void togglePause();
    // Ferme le fichier et revient à l'écran d'accueil
    Q_INVOKABLE void stop();
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
    void tracksChanged();
    void subDelayChanged();
    void eofReachedChanged();
    void mutedChanged();
    void infoChanged();
    void fileLoaded();
    void endOfFile();

private slots:
    void handleEvents();
    // Le contexte de rendu existe : charger le fichier demandé avant (sinon mpv n'a pas de sortie vidéo)
    void onRenderReady();

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
    QVariantList m_tracks;
    double m_subDelay = 0.0;
    bool m_eofReached = false;
    bool m_muted = false;
    QVariantMap m_info;
    bool m_renderReady = false;
    QUrl m_pendingLoad;
};

// Appelé depuis Rust avant la création de la fenêtre.
void lumen_init_video();
