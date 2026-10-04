#include "probe.h"

extern "C" {
#include <libavformat/avformat.h>
}

double lumen_probe_duration(const QString &path)
{
    AVFormatContext *ctx = nullptr;
    const QByteArray file = path.toUtf8();
    if (avformat_open_input(&ctx, file.constData(), nullptr, nullptr) < 0)
        return -1.0;

    // La plupart des conteneurs (MKV, MP4…) donnent la durée dans l'en-tête ;
    // sinon on analyse le début des flux, plus lent.
    if (ctx->duration == AV_NOPTS_VALUE || ctx->duration <= 0)
        avformat_find_stream_info(ctx, nullptr);

    const double seconds = ctx->duration != AV_NOPTS_VALUE && ctx->duration > 0
        ? static_cast<double>(ctx->duration) / AV_TIME_BASE
        : -1.0;
    avformat_close_input(&ctx);
    return seconds;
}
