#include "thumbnail.h"

#include <atomic>
#include <memory>

#include <QtCore/QRunnable>
#include <QtCore/QThreadPool>
#include <QtQuick/QQuickImageProvider>

extern "C" {
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libswscale/swscale.h>
}

QImage lumen_decode_thumbnail(const QString &path, double seconds, int width)
{
    AVFormatContext *fmt = nullptr;
    const QByteArray file = path.toUtf8();
    if (avformat_open_input(&fmt, file.constData(), nullptr, nullptr) < 0)
        return {};
    // Libère tout en sortie, quel que soit le chemin
    std::unique_ptr<AVFormatContext, void (*)(AVFormatContext *)> fmtGuard(
        fmt, [](AVFormatContext *f) { avformat_close_input(&f); });

    if (avformat_find_stream_info(fmt, nullptr) < 0)
        return {};
    const AVCodec *codec = nullptr;
    const int index = av_find_best_stream(fmt, AVMEDIA_TYPE_VIDEO, -1, -1, &codec, 0);
    if (index < 0 || !codec)
        return {};
    AVStream *stream = fmt->streams[index];
    // Pochette d'album : pas de miniature qui change avec la position
    if (stream->disposition & AV_DISPOSITION_ATTACHED_PIC)
        return {};

    AVCodecContext *ctx = avcodec_alloc_context3(codec);
    std::unique_ptr<AVCodecContext, void (*)(AVCodecContext *)> ctxGuard(
        ctx, [](AVCodecContext *c) { avcodec_free_context(&c); });
    if (!ctx || avcodec_parameters_to_context(ctx, stream->codecpar) < 0)
        return {};
    // Seules les images clés sont décodées : beaucoup plus rapide, précision suffisante pour un aperçu
    ctx->skip_frame = AVDISCARD_NONKEY;
    ctx->thread_count = 1;
    if (avcodec_open2(ctx, codec, nullptr) < 0)
        return {};

    int64_t target = av_rescale_q(static_cast<int64_t>(seconds * AV_TIME_BASE), AV_TIME_BASE_Q, stream->time_base);
    if (stream->start_time != AV_NOPTS_VALUE)
        target += stream->start_time;
    av_seek_frame(fmt, index, target, AVSEEK_FLAG_BACKWARD);

    AVPacket *packet = av_packet_alloc();
    AVFrame *frame = av_frame_alloc();
    std::unique_ptr<AVPacket, void (*)(AVPacket *)> packetGuard(packet, [](AVPacket *p) { av_packet_free(&p); });
    std::unique_ptr<AVFrame, void (*)(AVFrame *)> frameGuard(frame, [](AVFrame *f) { av_frame_free(&f); });

    bool decoded = false;
    // Borne de sécurité : un fichier abîmé ne doit pas faire lire tout le disque
    for (int read = 0; read < 600 && !decoded && av_read_frame(fmt, packet) >= 0; ++read) {
        if (packet->stream_index == index && avcodec_send_packet(ctx, packet) >= 0)
            decoded = avcodec_receive_frame(ctx, frame) >= 0;
        av_packet_unref(packet);
    }
    if (!decoded) {
        avcodec_send_packet(ctx, nullptr); // vider le décodeur (fin de fichier)
        decoded = avcodec_receive_frame(ctx, frame) >= 0;
    }
    if (!decoded || frame->width <= 0 || frame->height <= 0)
        return {};

    // Respecter le format d'affichage (pixels non carrés des DVD, par exemple)
    double aspect = static_cast<double>(frame->width) / frame->height;
    if (frame->sample_aspect_ratio.num > 0 && frame->sample_aspect_ratio.den > 0)
        aspect *= av_q2d(frame->sample_aspect_ratio);
    const int height = qMax(2, static_cast<int>(width / aspect) & ~1);

    SwsContext *sws = sws_getContext(frame->width, frame->height, static_cast<AVPixelFormat>(frame->format),
                                     width, height, AV_PIX_FMT_RGB32, SWS_BILINEAR, nullptr, nullptr, nullptr);
    if (!sws)
        return {};
    // AV_PIX_FMT_RGB32 (ARGB, boutisme natif) a la même disposition que QImage::Format_ARGB32
    QImage image(width, height, QImage::Format_ARGB32);
    uint8_t *dst[] = {image.bits()};
    const int dstStride[] = {static_cast<int>(image.bytesPerLine())};
    sws_scale(sws, frame->data, frame->linesize, 0, frame->height, dst, dstStride);
    sws_freeContext(sws);
    return image;
}

namespace {

// Réponse asynchrone, calculée dans le pool. Schéma de l'exemple officiel de Qt : Qt ne
// détruit la réponse qu'après finished(), émis ici depuis le thread de calcul (autorisé).
class ThumbnailResponse : public QQuickImageResponse, public QRunnable
{
public:
    ThumbnailResponse(QString path, double seconds, int width)
        : m_path(std::move(path)), m_seconds(seconds), m_width(width)
    {
        setAutoDelete(false);
    }

    void run() override
    {
        // La souris est déjà ailleurs : inutile de décoder
        if (!m_cancelled)
            m_image = lumen_decode_thumbnail(m_path, m_seconds, m_width);
        emit finished();
    }

    QQuickTextureFactory *textureFactory() const override
    {
        return QQuickTextureFactory::textureFactoryForImage(m_image);
    }

    void cancel() override { m_cancelled = true; }

private:
    QString m_path;
    double m_seconds;
    int m_width;
    QImage m_image;
    std::atomic_bool m_cancelled = false;
};

class ThumbnailProvider : public QQuickAsyncImageProvider
{
public:
    ThumbnailProvider()
    {
        // Deux décodages à la fois au plus : l'aperçu ne doit pas gêner la lecture
        m_pool.setMaxThreadCount(2);
    }

    // id : « <secondes>/<chemin UTF-8 en hexadécimal> »
    QQuickImageResponse *requestImageResponse(const QString &id, const QSize &requestedSize) override
    {
        const qsizetype slash = id.indexOf(u'/');
        const double seconds = id.left(slash).toDouble();
        const QString path = QString::fromUtf8(QByteArray::fromHex(id.mid(slash + 1).toLatin1()));
        const int width = requestedSize.width() > 0 ? requestedSize.width() : 320;
        auto *response = new ThumbnailResponse(path, seconds, width);
        m_pool.start(response);
        return response;
    }

private:
    QThreadPool m_pool;
};

} // namespace

void lumen_register_thumbnails(QQmlApplicationEngine &engine)
{
    engine.addImageProvider(QStringLiteral("thumbnail"), new ThumbnailProvider);
}

