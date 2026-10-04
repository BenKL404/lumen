// Utilitaires d'application appelés depuis Rust.
#pragma once

#include <QtCore/QString>
#include <QtQml/QQmlApplicationEngine>

// Nom et identifiant de l'application (« lumen » : relie la fenêtre à lumen.desktop et à
// son icône sous Wayland). À appeler avant de créer QGuiApplication.
void lumen_init_app();

// Icône des fenêtres (logo embarqué). À appeler après la création de QGuiApplication.
void lumen_set_window_icon();

// Vrai si l'interface QML a bien été créée (au moins un objet racine).
bool lumen_qml_loaded(const QQmlApplicationEngine &engine);

// Ouvre un fichier (passé en argument de la ligne de commande) via openUrl() de Main.qml.
void lumen_open_file(const QQmlApplicationEngine &engine, const QString &path);

// Outil de développement : enregistre une image de la fenêtre après `delayMs`, puis la ferme.
void lumen_snapshot(const QQmlApplicationEngine &engine, const QString &path, int delayMs);
