import QtQuick

// Menu de sélection des pistes audio ou des sous-titres, ouvert depuis la barre de contrôle
Rectangle {
    id: menu

    property var video
    property var utils
    property var theme
    property string kind: "audio" // "audio" ou "sub"
    readonly property var tracks: video.tracks.filter(t => t.type === kind)
    readonly property bool isSub: kind === "sub"

    signal addSubtitleRequested()
    signal audioSettingsRequested()
    signal searchSubtitlesRequested()
    // Sous-titres par reconnaissance vocale : lancer, ou annuler (génération en cours)
    signal generateSubtitlesRequested()
    property bool generating: false
    property int generationProgress: 0

    function select(id) {
        video.command(["set", isSub ? "sid" : "aid", String(id)])
    }

    width: 360
    // Hauteur limitée par la fenêtre (donnée par Main.qml) : au-delà, le contenu défile
    property real maxHeight: Number.POSITIVE_INFINITY
    height: Math.min(content.implicitHeight + 20, maxHeight)
    radius: theme.radius
    color: theme.surface
    border.color: menu.theme.subtle

    // Absorbe les clics : ne pas mettre en pause en cliquant dans le menu
    MouseArea { anchors.fill: parent }

    component Entry: Rectangle {
        id: entry

        property var theme
        property string label
        property string detail
        property bool selected: false
        signal clicked()

        width: parent.width
        height: 34
        radius: 8
        color: entryArea.containsMouse ? entry.theme.surfaceHover : "transparent"

        Rectangle {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
            width: 6
            height: 6
            radius: 3
            color: entry.theme.accent
            visible: entry.selected
        }
        Text {
            anchors { left: parent.left; leftMargin: 28; right: detailText.left; rightMargin: 8; verticalCenter: parent.verticalCenter }
            text: entry.label
            color: entry.selected ? entry.theme.accent : entry.theme.text
            font.pixelSize: 14
            elide: Text.ElideRight
        }
        Text {
            id: detailText
            anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            text: entry.detail
            color: entry.theme.muted
            font.pixelSize: 12
        }
        MouseArea {
            id: entryArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: entry.clicked()
        }
    }

    Flickable {
        id: scroller
        anchors { fill: parent; margins: 10 }
        contentHeight: content.implicitHeight
        clip: true
        interactive: contentHeight > height
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: content
            width: scroller.width
            spacing: 2

            Text {
                text: menu.isSub ? "Sous-titres" : "Pistes audio"
                color: menu.theme.muted
                font.pixelSize: 12
                font.weight: Font.Medium
                leftPadding: 12
                topPadding: 4
                bottomPadding: 6
            }

            Entry {
                theme: menu.theme
                visible: menu.isSub
                label: "Désactivés"
                selected: !menu.tracks.some(t => t.selected)
                onClicked: menu.video.command(["set", "sid", "no"])
            }

            Repeater {
                model: menu.tracks
                delegate: Entry {
                    theme: menu.theme
                    required property var modelData
                    label: menu.utils.trackLabel(modelData.id, modelData.title || "", modelData.lang || "",
                                                 modelData.codec || "", modelData["demux-channel-count"] || 0)
                    detail: modelData.external ? "Externe" : (modelData["default"] ? "Par défaut" : "")
                    selected: modelData.selected === true
                    onClicked: menu.select(modelData.id)
                }
            }

            Text {
                visible: !menu.isSub && menu.tracks.length === 0
                text: "Aucune piste audio"
                color: menu.theme.muted
                font.pixelSize: 14
                leftPadding: 28
                topPadding: 6
                bottomPadding: 6
            }

            // ------------------------------------- Audio : égaliseur, normalisation
            Rectangle {
                visible: !menu.isSub
                width: parent.width
                height: 1
                color: menu.theme.raised
            }
            Entry {
                theme: menu.theme
                visible: !menu.isSub
                label: "Égaliseur et normalisation…"
                onClicked: menu.audioSettingsRequested()
            }

            // ------------------------------------- Sous-titres : fichier, recherche, génération
            Rectangle {
                visible: menu.isSub
                width: parent.width
                height: 1
                color: menu.theme.raised
            }

            Entry {
                theme: menu.theme
                visible: menu.isSub
                label: "Ajouter un fichier de sous-titres…"
                onClicked: menu.addSubtitleRequested()
            }
            Entry {
                theme: menu.theme
                visible: menu.isSub
                label: "Rechercher en ligne…"
                onClicked: menu.searchSubtitlesRequested()
            }
            Entry {
                theme: menu.theme
                visible: menu.isSub
                label: menu.generating ? "Annuler la génération (" + menu.generationProgress + " %)"
                                       : "Générer (reconnaissance vocale)…"
                onClicked: menu.generateSubtitlesRequested()
            }

        }
    }
}
