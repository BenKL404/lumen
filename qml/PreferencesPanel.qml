import QtQuick

// Préférences (F5), à la manière de PotPlayer : catégories à gauche, réglages à droite,
// recherche dans tous les réglages. Chaque réglage est décrit une seule fois dans `rows`
// (catégorie, nom, description, type de contrôle, lecture et écriture de la valeur).
Rectangle {
    id: panel

    property var app        // fenêtre principale (Main.qml)
    property var theme
    property var settings
    property var playlist
    property var image
    property var sound
    property var history
    property var utils

    signal closeRequested()
    // Une valeur a changé : à enregistrer
    signal changed()

    property string category: "general"
    property string search: ""

    readonly property var categories: [
        { id: "general", label: "Général" },
        { id: "lecture", label: "Lecture" },
        { id: "video", label: "Vidéo" },
        { id: "audio", label: "Audio" },
        { id: "sous-titres", label: "Sous-titres" },
        { id: "apparence", label: "Apparence" },
        { id: "extensions", label: "Extensions" },
        { id: "raccourcis", label: "Raccourcis" }
    ]

    // « FR, en ;; » -> « fr,en » (même règle que language_list dans settings.rs)
    function languageList(text) {
        return text.split(/[\s,;]+/).map(c => c.toLowerCase()).filter(c => /^[a-z-]+$/.test(c)).join(",")
    }

    function categoryLabel(id) {
        return categories.find(c => c.id === id).label
    }

    // Types : toggle, choice (options), slider (min, max, unit), text (placeholder), button, keys
    // Relu à l'ouverture du dossier ou sur « Actualiser » (fichiers ajoutés entre-temps)
    property int extensionsVersion: 0

    readonly property var rows: {
        const s = settings, a = app, img = image, snd = sound, pl = playlist
        const imageRow = (key, label, min, max, neutral, unit) => ({
            cat: "video", label: label, type: "slider", min: min, max: max, neutral: neutral, unit: unit || "",
            get: () => img[key], set: (v) => img[key] = v
        })
        const list = [
            { cat: "general", label: "Une seule fenêtre Lumen", desc: "Les vidéos ouvertes s'affichent dans la fenêtre déjà ouverte",
              type: "toggle", get: () => s.singleInstance, set: (v) => s.singleInstance = v },
            { cat: "general", label: "Toujours au premier plan", desc: "La fenêtre reste au-dessus des autres (selon le bureau)",
              type: "toggle", get: () => a.pinned, set: (v) => a.pinned = v },
            { cat: "general", label: "Messages à l'écran", desc: "Volume, saut, vitesse… affichés sur la vidéo",
              type: "toggle", get: () => s.showOsd, set: (v) => s.showOsd = v },
            { cat: "general", label: "Fichier de paramètres", desc: "~/.config/lumen/settings.toml, modifiable à la main",
              type: "button", button: "Ouvrir", action: () => { a.saveSettings(); Qt.openUrlExternally(s.fileUrl()) } },

            { cat: "lecture", label: "Reprendre là où je me suis arrêté", desc: "Position, pistes et sous-titres de chaque vidéo",
              type: "toggle", get: () => s.resumePlayback, set: (v) => s.resumePlayback = v },
            { cat: "lecture", label: "Historique de lecture", desc: "Positions mémorisées (les signets sont conservés)",
              type: "button", button: "Effacer", action: () => { panel.history.clear(); a.osd("Historique de lecture effacé") } },
            { cat: "lecture", label: "Playlist automatique des épisodes", desc: "Ajoute les fichiers du même nom (épisodes, parties…)",
              type: "toggle", get: () => s.autoPlaylist, set: (v) => s.autoPlaylist = v },
            { cat: "lecture", label: "Répétition", type: "choice",
              options: [{ label: "Désactivée", value: 0 }, { label: "Le fichier", value: 1 }, { label: "La playlist", value: 2 }],
              get: () => pl.repeatMode, set: (v) => pl.setRepeat(v) },
            { cat: "lecture", label: "Lecture aléatoire", desc: "Chaque fichier de la playlist une fois, dans le désordre",
              type: "toggle", get: () => pl.shuffle, set: (v) => pl.setShuffleEnabled(v) },
            { cat: "lecture", label: "Saut court", desc: "Flèches gauche et droite", type: "slider", min: 1, max: 60, unit: " s",
              get: () => s.seekShort, set: (v) => s.seekShort = v },
            { cat: "lecture", label: "Saut long", desc: "Ctrl + flèches", type: "slider", min: 5, max: 300, unit: " s",
              get: () => s.seekLong, set: (v) => s.seekLong = v },

            { cat: "video", label: "Décodage matériel", desc: "Moins de processeur et de batterie (VA-API, NVDEC…)",
              type: "toggle", get: () => s.hardwareDecoding, set: (v) => s.hardwareDecoding = v },
            { cat: "video", label: "Agrandissement", desc: "Vidéos plus petites que l'écran (DVD, 720p…)", type: "choice",
              options: [{ label: "Standard", value: 0 }, { label: "Haute qualité", value: 1 }, { label: "FSR (AMD)", value: 2 }],
              get: () => s.upscaler, set: (v) => s.upscaler = v },
            { cat: "video", label: "Netteté", desc: "Accentue les détails (AMD CAS, sur la carte graphique)", type: "slider",
              min: 0, max: 100, neutral: 0, unit: " %", get: () => img.sharpness, set: (v) => img.sharpness = v },
            imageRow("brightness", "Luminosité", -100, 100, 0),
            imageRow("contrast", "Contraste", -100, 100, 0),
            imageRow("saturation", "Saturation", -100, 100, 0),
            imageRow("gamma", "Gamma", -100, 100, 0),
            imageRow("hue", "Teinte", -100, 100, 0),
            imageRow("zoom", "Zoom", 25, 400, 100, " %"),
            { cat: "video", label: "Réglages d'image", desc: "Revenir aux valeurs neutres", type: "button", button: "Réinitialiser",
              action: () => { ["brightness", "contrast", "saturation", "gamma", "hue", "sharpness"].forEach(k => img[k] = 0); img.zoom = 100 } },

            { cat: "audio", label: "Normaliser le volume", desc: "Dialogues plus audibles, explosions moins fortes",
              type: "toggle", get: () => snd.normalize, set: (v) => snd.normalize = v },
            { cat: "audio", label: "Égaliseur", desc: "Réglage fin bande par bande : touche E", type: "choice",
              options: panel.utils.equalizerPresets().map(name => ({ label: name, value: JSON.stringify(Array.from(panel.utils.equalizerPreset(name))) })),
              get: () => JSON.stringify(snd.equalizer), set: (v) => snd.equalizer = JSON.parse(v) },
            { cat: "audio", label: "Langues audio préférées", desc: "Codes séparés par des virgules, par ordre de préférence",
              type: "text", placeholder: "fr,en", get: () => s.audioLanguages, set: (v) => s.audioLanguages = panel.languageList(v) },

            { cat: "sous-titres", label: "Langues de sous-titres préférées", desc: "Codes séparés par des virgules ; vide : selon le fichier",
              type: "text", placeholder: "fr,en", get: () => s.subtitleLanguages, set: (v) => s.subtitleLanguages = panel.languageList(v) },
            { cat: "sous-titres", label: "Clé d'API OpenSubtitles", desc: "Pour la recherche en ligne (Ctrl+J) ; gratuite sur opensubtitles.com",
              type: "text", placeholder: "Coller la clé ici", get: () => s.opensubtitlesApiKey, set: (v) => s.opensubtitlesApiKey = v.trim() },
            { cat: "sous-titres", label: "Obtenir une clé", desc: "Compte gratuit › profil › API consumers", type: "button", button: "Ouvrir le site",
              action: () => Qt.openUrlExternally("https://www.opensubtitles.com/consumers") },
            { cat: "sous-titres", label: "Taille des sous-titres", type: "slider", min: 50, max: 300, neutral: 100, unit: " %",
              get: () => s.subtitleScale, set: (v) => s.subtitleScale = v },

            { cat: "apparence", label: "Thème", desc: "Sombre, clair, OLED, ou un skin de ton dossier de skins", type: "choice",
              options: JSON.parse(panel.utils.themes()).map(t => ({ label: t.label, value: t.id })),
              get: () => s.theme, set: (v) => s.theme = v },
            { cat: "apparence", label: "Couleur d'accent", desc: "Boutons actifs, progression, sélection", type: "swatches",
              options: [{ label: "Thème", value: "" }, { value: "#FF8C1A" }, { value: "#F2A541" }, { value: "#E5484D" },
                        { value: "#E93D82" }, { value: "#8E4EC6" }, { value: "#2D7FF9" }, { value: "#12A594" }, { value: "#46A758" }],
              get: () => s.accent, set: (v) => s.accent = v },
            { cat: "apparence", label: "Couleur personnalisée", desc: "Code hexadécimal, par exemple #3FA9F5", type: "text",
              placeholder: "#RRGGBB", get: () => s.accent, set: (v) => s.accent = /^#[0-9a-fA-F]{6}$/.test(v.trim()) ? v.trim() : s.accent },
            { cat: "apparence", label: "Taille de l'interface", desc: "Appliquée au prochain lancement de Lumen", type: "choice",
              options: [{ label: "Compacte", value: 90 }, { label: "Normale", value: 100 }, { label: "Grande", value: 115 }, { label: "Très grande", value: 130 }],
              get: () => s.uiScale, set: (v) => s.uiScale = v },
            { cat: "apparence", label: "Skins", desc: "Fichiers JSON partageables : couleurs d'un thème (exemple fourni)", type: "button",
              button: "Ouvrir le dossier", action: () => Qt.openUrlExternally(panel.utils.skinsFolderUrl()) },

            { cat: "raccourcis", label: "Modifier les raccourcis", desc: "Section [raccourcis] de settings.toml",
              type: "button", button: "Modifier…", action: () => { a.saveSettings(); Qt.openUrlExternally(s.fileUrl()) } },
            { cat: "raccourcis", label: "Appliquer les modifications", desc: "Relit le fichier sans relancer Lumen",
              type: "button", button: "Recharger", action: () => { s.reloadShortcuts(); a.osd("Raccourcis rechargés") } }
        ]
        extensionsVersion
        const extensionRows = [
            { cat: "extensions", label: "Dossier des extensions",
              desc: "Scripts mpv (.lua, .js) dans ~/.config/lumen/scripts",
              type: "button", button: "Ouvrir", action: () => { panel.utils.prepareExtensionsFolder(); Qt.openUrlExternally(panel.utils.extensionsFolderUrl()) } },
            { cat: "extensions", label: "Scripts ajoutés au dossier", desc: "Relire la liste ; désactiver un script prend effet au prochain lancement",
              type: "button", button: "Actualiser", action: () => panel.extensionsVersion++ }
        ].concat(JSON.parse(panel.utils.extensions(s.disabledExtensions)).map(ext => ({
            cat: "extensions", label: ext.name, desc: ext.description || ext.file, type: "toggle",
            get: () => Array.from(s.disabledExtensions).indexOf(ext.file) < 0,
            set: (on) => {
                const disabled = Array.from(s.disabledExtensions).filter(f => f !== ext.file)
                if (!on)
                    disabled.push(ext.file)
                s.disabledExtensions = disabled
                if (on)
                    a.loadExtension(ext.path)
            }
        })))
        return list.concat(extensionRows, s.shortcutActions().map(id => ({
            cat: "raccourcis", label: s.actionLabel(id), type: "keys", action_id: id
        })))
    }

    readonly property var visibleRows: {
        const query = search.trim().toLowerCase()
        if (query === "")
            return rows.filter(r => r.cat === category)
        return rows.filter(r => (r.label + " " + (r.desc || "")).toLowerCase().indexOf(query) >= 0)
    }

    width: 820
    height: 560
    radius: theme.radius
    color: theme.menu
    border.color: theme.border
    clip: true

    // Absorbe les clics : ne pas fermer en cliquant dans la fenêtre
    MouseArea { anchors.fill: parent }

    // ------------------------------------------------------------ Barre latérale
    Rectangle {
        id: sidebar
        anchors { left: parent.left; top: parent.top; bottom: parent.bottom }
        width: 210
        color: panel.theme.shade

        Text {
            id: title
            anchors { left: parent.left; top: parent.top; leftMargin: 20; topMargin: 20 }
            text: "Préférences"
            color: panel.theme.text
            font.pixelSize: 17
            font.weight: Font.DemiBold
        }

        // Recherche dans tous les réglages
        Rectangle {
            id: searchBox
            anchors { left: parent.left; right: parent.right; top: title.bottom; margins: 14; topMargin: 16 }
            height: 32
            radius: 6
            color: panel.theme.field
            border.color: searchField.activeFocus ? panel.theme.accent : panel.theme.border

            Icon {
                id: searchIcon
                anchors { left: parent.left; leftMargin: 9; verticalCenter: parent.verticalCenter }
                name: "search"
                size: 14
                color: panel.theme.muted
            }
            TextInput {
                id: searchField
                anchors { left: searchIcon.right; right: parent.right; leftMargin: 7; rightMargin: 9; verticalCenter: parent.verticalCenter }
                color: panel.theme.text
                selectionColor: panel.theme.accent
                font.pixelSize: 13
                clip: true
                onTextChanged: panel.search = text
                Keys.onEscapePressed: text === "" ? panel.closeRequested() : text = ""
            }
            Text {
                anchors { left: searchField.left; verticalCenter: parent.verticalCenter }
                visible: searchField.text === ""
                text: "Rechercher"
                color: panel.theme.muted
                font.pixelSize: 13
            }
        }

        Column {
            anchors { left: parent.left; right: parent.right; top: searchBox.bottom; margins: 10; topMargin: 14 }
            spacing: 2

            Repeater {
                model: panel.categories
                delegate: Rectangle {
                    id: categoryItem
                    required property var modelData
                    readonly property bool current: panel.search === "" && panel.category === modelData.id

                    width: parent.width
                    height: 36
                    radius: 8
                    // Catégorie en cours : icône et texte en orange suffisent, sans fond
                    color: !current && categoryArea.containsMouse ? panel.theme.surfaceHover : "transparent"

                    Text {
                        anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                        text: categoryItem.modelData.label
                        color: categoryItem.current ? panel.theme.accent : panel.theme.text
                        font.pixelSize: 13
                        font.weight: categoryItem.current ? Font.Medium : Font.Normal
                    }
                    MouseArea {
                        id: categoryArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            searchField.text = ""
                            panel.category = categoryItem.modelData.id
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ Réglages
    Text {
        id: pageTitle
        anchors { left: sidebar.right; top: parent.top; leftMargin: 28; topMargin: 22 }
        text: panel.search.trim() !== "" ? "Résultats pour « " + panel.search.trim() + " »" : panel.categoryLabel(panel.category)
        color: panel.theme.text
        font.pixelSize: 16
        font.weight: Font.DemiBold
    }
    IconButton {
        anchors { right: parent.right; rightMargin: 12; verticalCenter: pageTitle.verticalCenter }
        implicitHeight: 32
        theme: panel.theme
        icon: "x"
        iconSize: 15
        onClicked: panel.closeRequested()
    }

    Flickable {
        id: page
        anchors { left: sidebar.right; right: parent.right; top: pageTitle.bottom; bottom: parent.bottom
                  leftMargin: 16; rightMargin: 16; topMargin: 14; bottomMargin: 12 }
        clip: true
        contentHeight: rowsColumn.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: rowsColumn
            width: page.width

            Text {
                visible: panel.visibleRows.length === 0
                width: parent.width
                topPadding: 30
                horizontalAlignment: Text.AlignHCenter
                text: "Aucun réglage trouvé"
                color: panel.theme.muted
                font.pixelSize: 13
            }

            Repeater {
                model: panel.visibleRows

                delegate: Item {
                    id: row

                    required property var modelData
                    readonly property var r: modelData

                    width: rowsColumn.width
                    height: r.type === "keys" ? 36 : (r.desc || panel.search !== "" ? 60 : 48)

                    Rectangle {
                        anchors { left: parent.left; right: parent.right; bottom: parent.bottom; leftMargin: 12; rightMargin: 12 }
                        height: 1
                        color: panel.theme.border
                        visible: row.r.type !== "keys"
                    }

                    Column {
                        anchors { left: parent.left; leftMargin: 12; right: control.left; rightMargin: 16; verticalCenter: parent.verticalCenter }
                        spacing: 3

                        Text {
                            width: parent.width
                            text: row.r.label
                            color: panel.theme.text
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                        Text {
                            width: parent.width
                            visible: text !== ""
                            // En recherche, la catégorie aide à s'y retrouver
                            text: (panel.search !== "" ? panel.categoryLabel(row.r.cat) + (row.r.desc ? " · " : "") : "") + (row.r.desc || "")
                            color: panel.theme.muted
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }
                    }

                    // ---------------------------------------- Contrôles
                    Component {
                        id: toggleControl
                        Rectangle {
                            readonly property bool on: row.r.get()
                            width: 40
                            height: 22
                            radius: 11
                            color: on ? panel.theme.accent : panel.theme.track
                            Behavior on color { ColorAnimation { duration: 120 } }
                            Rectangle {
                                y: 3
                                x: parent.on ? parent.width - width - 3 : 3
                                width: 16
                                height: 16
                                radius: 8
                                color: "#FFFFFF"
                                Behavior on x { NumberAnimation { duration: 120 } }
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { row.r.set(!parent.on); panel.changed() }
                            }
                        }
                    }

                    Component {
                        id: choiceControl
                        Row {
                            spacing: 4
                            Repeater {
                                model: row.r.options
                                delegate: Rectangle {
                                    required property var modelData
                                    readonly property bool selected: row.r.get() === modelData.value
                                    width: optionText.implicitWidth + 18
                                    height: 26
                                    radius: 13
                                    color: selected ? Qt.rgba(panel.theme.accent.r, panel.theme.accent.g, panel.theme.accent.b, 0.18)
                                         : optionArea.containsMouse ? panel.theme.surfaceHover : panel.theme.subtle
                                    border.color: selected ? panel.theme.accent : "transparent"
                                    Text {
                                        id: optionText
                                        anchors.centerIn: parent
                                        text: parent.modelData.label
                                        color: parent.selected ? panel.theme.accent : panel.theme.text
                                        font.pixelSize: 12
                                    }
                                    MouseArea {
                                        id: optionArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: { row.r.set(parent.modelData.value); panel.changed() }
                                    }
                                }
                            }
                        }
                    }

                    Component {
                        id: swatchesControl
                        Row {
                            spacing: 6
                            Repeater {
                                model: row.r.options
                                delegate: Rectangle {
                                    required property var modelData
                                    readonly property bool selected: row.r.get() === modelData.value
                                    // Pastille de couleur, ou « Thème » (accent du thème)
                                    width: modelData.label ? themeText.implicitWidth + 16 : 24
                                    height: 24
                                    radius: 12
                                    color: modelData.value !== "" ? modelData.value : panel.theme.subtle
                                    border.width: selected ? 2 : 1
                                    border.color: selected ? panel.theme.text : panel.theme.border
                                    Text {
                                        id: themeText
                                        anchors.centerIn: parent
                                        visible: parent.modelData.label !== undefined
                                        text: parent.modelData.label || ""
                                        color: panel.theme.text
                                        font.pixelSize: 11
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: { row.r.set(parent.modelData.value); panel.changed() }
                                    }
                                }
                            }
                        }
                    }

                    Component {
                        id: sliderControl
                        Row {
                            spacing: 12
                            SeekBar {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 200
                                theme: panel.theme
                                live: true
                                to: row.r.max - row.r.min
                                value: row.r.get() - row.r.min
                                origin: (row.r.neutral !== undefined ? row.r.neutral : row.r.min) - row.r.min
                                onMoved: (v) => { row.r.set(Math.round(v + row.r.min)); panel.changed() }
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 52
                                horizontalAlignment: Text.AlignRight
                                text: (row.r.unit === "" && row.r.min < 0 && row.r.get() > 0 ? "+" : "") + row.r.get() + row.r.unit
                                color: row.r.neutral !== undefined && row.r.get() !== row.r.neutral ? panel.theme.accent : panel.theme.text
                                font.pixelSize: 12
                                font.family: "monospace"
                            }
                        }
                    }

                    Component {
                        id: textControl
                        Rectangle {
                            width: 200
                            height: 30
                            radius: 6
                            color: panel.theme.field
                            border.color: input.activeFocus ? panel.theme.accent : panel.theme.border
                            TextInput {
                                id: input
                                anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
                                verticalAlignment: TextInput.AlignVCenter
                                color: panel.theme.text
                                selectionColor: panel.theme.accent
                                font.pixelSize: 13
                                clip: true
                                Component.onCompleted: text = row.r.get()
                                onEditingFinished: { row.r.set(text); text = row.r.get(); panel.changed() }
                            }
                            Text {
                                anchors { left: parent.left; leftMargin: 10; verticalCenter: parent.verticalCenter }
                                visible: input.text === "" && !input.activeFocus
                                text: row.r.placeholder || ""
                                color: panel.theme.muted
                                font.pixelSize: 13
                            }
                        }
                    }

                    Component {
                        id: buttonControl
                        Rectangle {
                            width: buttonText.implicitWidth + 24
                            height: 30
                            radius: 6
                            color: buttonArea.containsMouse ? panel.theme.surfaceHover : panel.theme.subtle
                            border.color: panel.theme.border
                            Text {
                                id: buttonText
                                anchors.centerIn: parent
                                text: row.r.button
                                color: panel.theme.text
                                font.pixelSize: 12
                            }
                            MouseArea {
                                id: buttonArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: { row.r.action(); panel.changed() }
                            }
                        }
                    }

                    Component {
                        id: keysControl
                        Row {
                            spacing: 6
                            readonly property var keys: { panel.settings.shortcutsVersion; return panel.settings.keys(row.r.action_id) }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: parent.keys.length === 0
                                text: "—"
                                color: panel.theme.muted
                                font.pixelSize: 13
                            }
                            Repeater {
                                model: parent.keys
                                delegate: Rectangle {
                                    required property string modelData
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: Math.max(24, keyText.implicitWidth + 14)
                                    height: 22
                                    radius: 4
                                    color: panel.theme.subtle
                                    border.color: panel.theme.strong
                                    Text {
                                        id: keyText
                                        anchors.centerIn: parent
                                        text: panel.app.formatKey(parent.modelData)
                                        color: panel.theme.text
                                        font.pixelSize: 12
                                    }
                                }
                            }
                        }
                    }

                    Loader {
                        id: control
                        anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
                        sourceComponent: ({
                            "toggle": toggleControl, "choice": choiceControl, "slider": sliderControl, "swatches": swatchesControl,
                            "text": textControl, "button": buttonControl, "keys": keysControl
                        })[row.r.type]
                    }
                }
            }
        }
    }
}
