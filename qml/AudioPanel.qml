import QtQuick

// Son : normalisation du volume et égaliseur 10 bandes.
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

    width: 440
    // Hauteur limitée par la fenêtre (donnée par Main.qml) : au-delà, le contenu défile
    property real maxHeight: Number.POSITIVE_INFINITY
    height: Math.min(content.implicitHeight + 24, maxHeight)
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

        // Même style sobre que les choix des préférences : fond plus clair pour l'option active
        width: chipText.implicitWidth + 20
        height: 26
        radius: 6
        color: selected ? theme.raised : chipArea.containsMouse ? theme.subtle : "transparent"
        border.color: selected ? "transparent" : theme.border

        Text {
            id: chipText
            anchors.centerIn: parent
            text: chip.label
            color: chip.selected || chipArea.containsMouse ? chip.theme.text : chip.theme.muted
            font.pixelSize: 12
            font.weight: chip.selected ? Font.DemiBold : Font.Normal
        }
        MouseArea {
            id: chipArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: chip.clicked()
        }
    }

    Flickable {
        id: scroller
        anchors { fill: parent; margins: 12 }
        contentHeight: content.implicitHeight
        clip: true
        interactive: contentHeight > height
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: content
            width: scroller.width
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

        }
    }
}
