import QtQuick

// Curseur générique : progression de lecture ou volume.
Item {
    id: slider

    property var theme
    property real value: 0
    property real to: 1
    // true : émet moved() pendant le glissement (volume)
    // false : n'émet qu'au relâchement (position, évite de saturer mpv)
    property bool live: false
    // Point de départ du remplissage (réglages centrés sur une valeur neutre)
    property real origin: 0
    property var tooltipFormatter: null
    // Repères (barre de progression) : chapitres [{ title, time }], signets [secondes],
    // boucle A-B (bornes en secondes, -1 si non définies)
    property var chapters: []
    property var bookmarks: []
    property real loopA: -1
    property real loopB: -1
    // Aperçu au survol : fonction (secondes) -> adresse d'image, ou null
    property var previewSource: null

    signal moved(real value)

    readonly property real shownValue: area.pressed ? dragValue : value
    property real dragValue: 0

    implicitHeight: 20
    opacity: enabled ? 1 : 0.4

    function valueAt(x) {
        return Math.max(0, Math.min(1, x / width)) * to
    }

    function xAt(seconds) {
        return to > 0 ? width * Math.max(0, Math.min(1, seconds / to)) : 0
    }

    // Titre du chapitre qui contient `seconds`
    function chapterAt(seconds) {
        let title = ""
        for (const c of chapters) {
            if (c.time <= seconds)
                title = c.title || ""
        }
        return title
    }

    Rectangle {
        id: track
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: area.containsMouse || area.pressed ? 6 : 4
        radius: height / 2
        color: slider.theme.track
        Behavior on height { NumberAnimation { duration: 120 } }

        Rectangle {
            x: Math.min(slider.xAt(slider.origin), slider.xAt(slider.shownValue))
            width: Math.abs(slider.xAt(slider.shownValue) - slider.xAt(slider.origin))
            height: parent.height
            radius: parent.radius
            color: slider.theme.accent
        }

        // Boucle A-B : zone entre A et B (ou simple trait tant que B n'est pas défini)
        Rectangle {
            visible: slider.loopA >= 0
            x: slider.xAt(slider.loopA)
            width: slider.loopB > slider.loopA ? slider.xAt(slider.loopB) - x : 2
            y: -2
            height: parent.height + 4
            radius: 2
            color: Qt.rgba(slider.theme.accent.r, slider.theme.accent.g, slider.theme.accent.b, 0.35)
            border.color: slider.theme.accent
            border.width: 1
        }

        // Chapitres : fines coupures dans la barre
        Repeater {
            model: slider.chapters
            delegate: Rectangle {
                required property var modelData
                visible: modelData.time > 0
                x: slider.xAt(modelData.time) - 1
                width: 2
                height: parent.height
                color: slider.theme.chrome
            }
        }
    }

    // Signets : petits losanges au-dessus de la barre
    Repeater {
        model: slider.bookmarks
        delegate: Rectangle {
            required property real modelData
            x: slider.xAt(modelData) - width / 2
            y: track.y - 9
            width: 7
            height: 7
            rotation: 45
            radius: 1
            color: slider.theme.text
        }
    }

    Rectangle {
        width: 14
        height: 14
        radius: 7
        color: slider.theme.text
        anchors.verticalCenter: parent.verticalCenter
        x: (slider.to > 0 ? slider.width * Math.min(1, slider.shownValue / slider.to) : 0) - width / 2
        scale: area.containsMouse || area.pressed ? 1 : 0
        Behavior on scale { NumberAnimation { duration: 120 } }
    }

    // ------------------------------------------------- Aperçu au survol
    readonly property real hoverSeconds: valueAt(area.mouseX)
    property string previewRequested: ""
    // Dernière miniature prête : reste affichée pendant le calcul de la suivante
    property string previewShown: ""

    onHoverSecondsChanged: if (area.containsMouse && previewSource) previewTimer.restart()

    // Petit délai : pas de calcul pour chaque pixel parcouru par la souris
    Timer {
        id: previewTimer
        interval: 40
        onTriggered: slider.previewRequested = slider.previewSource ? slider.previewSource(slider.hoverSeconds) : ""
    }

    Image {
        visible: false
        asynchronous: true
        source: slider.previewRequested
        sourceSize.width: 320
        onStatusChanged: if (status === Image.Ready) slider.previewShown = source
    }

    Rectangle {
        id: previewBox
        visible: slider.previewSource !== null && area.containsMouse && slider.previewShown !== ""
                 && previewImage.status === Image.Ready && previewImage.implicitWidth > 0
        width: 184
        height: previewImage.implicitWidth > 0 ? (width - 4) * previewImage.implicitHeight / previewImage.implicitWidth + 4 : 0
        x: Math.max(0, Math.min(slider.width - width, area.mouseX - width / 2))
        y: tooltip.y - height - 6
        radius: 8
        color: "#000000"
        border.color: slider.theme.border

        Image {
            id: previewImage
            anchors { fill: parent; margins: 2 }
            source: slider.previewShown
            sourceSize.width: 320
            fillMode: Image.PreserveAspectFit
            smooth: true
        }
    }

    // Info-bulle temporelle au survol
    Rectangle {
        id: tooltip
        visible: slider.tooltipFormatter !== null && area.containsMouse && slider.to > 0
        y: -height - 10
        x: Math.max(0, Math.min(slider.width - width, area.mouseX - width / 2))
        width: tip.implicitWidth + 16
        height: 26
        radius: 8
        color: slider.theme.surface

        Text {
            id: tip
            anchors.centerIn: parent
            text: {
                if (!slider.tooltipFormatter)
                    return ""
                const seconds = slider.valueAt(area.mouseX)
                const chapter = slider.chapterAt(seconds)
                return slider.tooltipFormatter(seconds) + (chapter !== "" ? "  ·  " + chapter : "")
            }
            color: slider.theme.text
            font.pixelSize: 12
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        anchors.topMargin: -6
        anchors.bottomMargin: -6
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor

        onPressed: (mouse) => {
            slider.dragValue = slider.valueAt(mouse.x)
            if (slider.live) slider.moved(slider.dragValue)
        }
        onPositionChanged: (mouse) => {
            if (!pressed) return
            slider.dragValue = slider.valueAt(mouse.x)
            if (slider.live) slider.moved(slider.dragValue)
        }
        onReleased: slider.moved(slider.dragValue)
    }
}
