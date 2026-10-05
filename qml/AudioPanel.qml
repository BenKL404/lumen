import QtQuick

// Son : normalisation du volume, égaliseur 10 bandes et décalage audio.
// Les valeurs vivent dans `sound` (Main.qml), qui les applique à mpv et les mémorise.
Rectangle {
    id: panel

    property var theme
    property var sound
    property var utils

    signal closeRequested()

    readonly property var bandLabels: ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"]

    function setBand(index, value) {
        const gains = sound.equalizer.slice()
        gains[index] = value
        sound.equalizer = gains
    }

    function shiftDelay(step) {
        sound.delay = Math.round((sound.delay + step) * 10) / 10
    }

    width: 440
    height: content.implicitHeight + 24
    radius: theme.radius
    color: theme.surface
    border.color: theme.border

    // Absorbe les clics : ne pas mettre en pause en cliquant dans le panneau
    MouseArea { anchors.fill: parent }

    // Curseur vertical d'une bande : de -12 à +12 dB, double-clic pour revenir à 0
    component EqSlider: Item {
        id: eq

        property var theme
        property int value: 0
        readonly property real travel: height / 2 - 7
        function yAt(v) { return height / 2 - v / 12 * travel }
        signal moved(int value)

        width: 30
        height: 120

        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            width: 4
            height: parent.height
            radius: 2
            color: eq.theme.track
        }
        // Repère du zéro
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            y: parent.height / 2
            width: 12
            height: 1
            color: eq.theme.muted
        }
        // Remplissage depuis le zéro
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            y: Math.min(eq.yAt(eq.value), parent.height / 2)
            width: 4
            height: Math.abs(eq.yAt(eq.value) - parent.height / 2)
            radius: 2
            color: eq.theme.accent
        }
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            y: eq.yAt(eq.value) - height / 2
            width: 14
            height: 14
            radius: 7
            color: eqArea.pressed || eqArea.containsMouse ? eq.theme.text : eq.theme.text
        }
        MouseArea {
            id: eqArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            function valueAt(y) {
                return Math.max(-12, Math.min(12, Math.round((eq.height / 2 - y) / eq.travel * 12)))
            }
            onPressed: (mouse) => eq.moved(valueAt(mouse.y))
            onPositionChanged: (mouse) => { if (pressed) eq.moved(valueAt(mouse.y)) }
            onDoubleClicked: eq.moved(0)
        }
    }

    // Petit bouton texte (préréglages)
    component Chip: Rectangle {
        id: chip
        property var theme
        property string label
        property bool selected: false
        signal clicked()

        width: chipText.implicitWidth + 18
        height: 26
        radius: 13
        color: selected ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.18)
             : chipArea.containsMouse ? theme.surfaceHover : theme.subtle
        border.color: selected ? theme.accent : "transparent"

        Text {
            id: chipText
            anchors.centerIn: parent
            text: chip.label
            color: chip.selected ? chip.theme.accent : chip.theme.text
            font.pixelSize: 12
        }
        MouseArea {
            id: chipArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: chip.clicked()
        }
    }

    Column {
        id: content
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 12 }
        spacing: 10

        // En-tête
        Item {
            width: parent.width
            height: 28

            Text {
                anchors { left: parent.left; leftMargin: 6; verticalCenter: parent.verticalCenter }
                text: "Son"
                color: panel.theme.muted
                font.pixelSize: 12
                font.weight: Font.Medium
            }
            IconButton {
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                implicitHeight: 28
                theme: panel.theme
                icon: "x"
                iconSize: 14
                onClicked: panel.closeRequested()
            }
        }

        // Normalisation : interrupteur
        Item {
            width: parent.width
            height: 40

            Column {
                anchors { left: parent.left; leftMargin: 6; verticalCenter: parent.verticalCenter }
                spacing: 2
                Text {
                    text: "Normaliser le volume"
                    color: panel.theme.text
                    font.pixelSize: 13
                }
                Text {
                    text: "Dialogues plus audibles, explosions moins fortes"
                    color: panel.theme.muted
                    font.pixelSize: 11
                }
            }
            Rectangle {
                id: toggle
                anchors { right: parent.right; rightMargin: 6; verticalCenter: parent.verticalCenter }
                width: 40
                height: 22
                radius: 11
                color: panel.sound.normalize ? panel.theme.accent : panel.theme.track
                Behavior on color { ColorAnimation { duration: 120 } }

                Rectangle {
                    y: 3
                    x: panel.sound.normalize ? parent.width - width - 3 : 3
                    width: 16
                    height: 16
                    radius: 8
                    color: "#FFFFFF"
                    Behavior on x { NumberAnimation { duration: 120 } }
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.sound.normalize = !panel.sound.normalize
                }
            }
        }

        Rectangle { width: parent.width; height: 1; color: panel.theme.border }

        // Égaliseur : préréglages
        Item {
            width: parent.width
            height: 26

            Text {
                anchors { left: parent.left; leftMargin: 6; verticalCenter: parent.verticalCenter }
                text: "Égaliseur"
                color: panel.theme.text
                font.pixelSize: 13
            }
            Row {
                anchors { right: parent.right; rightMargin: 6; verticalCenter: parent.verticalCenter }
                spacing: 5

                Repeater {
                    model: panel.utils.equalizerPresets()
                    delegate: Chip {
                        required property string modelData
                        theme: panel.theme
                        label: modelData
                        readonly property var gains: Array.from(panel.utils.equalizerPreset(modelData))
                        selected: JSON.stringify(gains) === JSON.stringify(panel.sound.equalizer)
                        onClicked: panel.sound.equalizer = gains
                    }
                }
            }
        }

        // Égaliseur : bandes
        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 10

            Repeater {
                model: 10
                delegate: Column {
                    id: band
                    required property int index
                    readonly property int gain: panel.sound.equalizer[index] || 0
                    spacing: 4

                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: (band.gain > 0 ? "+" : "") + band.gain
                        color: band.gain !== 0 ? panel.theme.accent : panel.theme.muted
                        font.pixelSize: 10
                        font.family: "monospace"
                    }
                    EqSlider {
                        theme: panel.theme
                        value: band.gain
                        onMoved: (v) => panel.setBand(band.index, v)
                    }
                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: panel.bandLabels[band.index]
                        color: panel.theme.muted
                        font.pixelSize: 10
                    }
                }
            }
        }

        Rectangle { width: parent.width; height: 1; color: panel.theme.border }

        // Décalage audio
        Item {
            width: parent.width
            height: 34

            Text {
                anchors { left: parent.left; leftMargin: 6; verticalCenter: parent.verticalCenter }
                text: "Décalage audio"
                color: panel.theme.text
                font.pixelSize: 13
            }
            Row {
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                spacing: 2

                IconButton { theme: panel.theme; icon: "minus"; onClicked: panel.shiftDelay(-0.1) }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 64
                    horizontalAlignment: Text.AlignHCenter
                    text: (panel.sound.delay > 0 ? "+" : "") + panel.sound.delay.toFixed(1) + " s"
                    color: panel.sound.delay !== 0 ? panel.theme.accent : panel.theme.text
                    font.pixelSize: 13
                    font.family: "monospace"
                }
                IconButton { theme: panel.theme; icon: "plus"; onClicked: panel.shiftDelay(0.1) }
                IconButton { theme: panel.theme; icon: "rotate-ccw"; iconSize: 16; onClicked: panel.sound.delay = 0 }
            }
        }
    }
}
