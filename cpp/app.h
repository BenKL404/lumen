// Utilitaires d'application appelés depuis Rust.
#pragma once

#include <QtQml/QQmlApplicationEngine>

// Vrai si l'interface QML a bien été créée (au moins un objet racine).
bool lumen_qml_loaded(const QQmlApplicationEngine &engine);
