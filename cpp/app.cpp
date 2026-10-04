#include "app.h"

bool lumen_qml_loaded(const QQmlApplicationEngine &engine)
{
    return !engine.rootObjects().isEmpty();
}
