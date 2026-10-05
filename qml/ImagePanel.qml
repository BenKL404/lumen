import QtQuick

// Réglages d'image : luminosité, contraste, saturation, gamma, teinte et zoom.
// Les valeurs vivent dans `image` (Main.qml), qui les applique à mpv et les mémorise.
Rectangle {
    id: panel

    property var theme
    property var image

    signal closeRequested()

    // Réglages affichés : clé dans `image`, bornes, valeur neutre
    readonly property var rows: [
        { key: "brightness", label: "Luminosité", min: -100, max: 100, neutral: 0 },
        { key: "contrast", label: "Contraste", min: -100, max: 100, neutral: 0 },
        { key: "saturation", label: "Saturation", min: -100, max: 100, neutral: 0 },
        { key: "gamma", label: "Gamma", min: -100, max: 100, neutral: 0 },
        { key: "hue", label: "Teinte", min: -100, max: 100, neutral: 0 },
        { key: "zoom", label: "Zoom", min: 25, max: 400, neutral: 100, unit: " %" },
        { key: "sharpness", label: "Netteté", min: 0, max: 100, neutral: 0, unit: " %" }
    ]

    readonly property bool modified: rows.some(r => image[r.key] !== r.neutral)

    width: 340
    height: content.implicitHeight + 20
    radius: theme.radius
    color: theme.surface
    border.color: theme.border

    // Absorbe les clics : ne pas mettre en pause en cliquant dans le panneau
    MouseArea { anchors.fill: parent }

    Column {
        id: content
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 10 }
        spacing: 2

        Item {
            width: parent.width
            height: 30

            Text {
                anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
                text: "Réglages d'image"
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

        Repeater {
            model: panel.rows

            delegate: Item {
                id: row

                required property var modelData
                readonly property int value: panel.image[modelData.key]
                readonly property bool changed: value !== modelData.neutral

                width: content.width
                height: 34

                // Double-clic sur le nom : valeur neutre
                Text {
                    id: name
                    anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
                    width: 82
                    text: row.modelData.label
                    color: row.changed ? panel.theme.text : panel.theme.muted
                    font.pixelSize: 13

                    MouseArea {
                        anchors.fill: parent
                        onDoubleClicked: panel.image[row.modelData.key] = row.modelData.neutral
                    }
                }
                SeekBar {
                    anchors { left: name.right; right: valueText.left; rightMargin: 10; verticalCenter: parent.verticalCenter }
                    theme: panel.theme
                    live: true
                    // SeekBar va de 0 à `to` : décalage pour les bornes négatives
                    to: row.modelData.max - row.modelData.min
                    value: row.value - row.modelData.min
                    origin: row.modelData.neutral - row.modelData.min
                    onMoved: (v) => panel.image[row.modelData.key] = Math.round(v + row.modelData.min)
                }
                Text {
                    id: valueText
                    anchors { right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter }
                    width: 46
                    horizontalAlignment: Text.AlignRight
                    text: (row.modelData.unit ? row.value + row.modelData.unit
                                              : (row.value > 0 ? "+" : "") + row.value)
                    color: row.changed ? panel.theme.accent : panel.theme.muted
                    font.pixelSize: 12
                    font.family: "monospace"
                }
            }
        }

        Item {
            width: parent.width
            height: 38

            Rectangle {
                anchors { right: parent.right; rightMargin: 4; verticalCenter: parent.verticalCenter }
                width: resetRow.implicitWidth + 20
                height: 28
                radius: 6
                opacity: panel.modified ? 1 : 0.4
                color: resetArea.containsMouse && panel.modified ? panel.theme.surfaceHover : panel.theme.subtle

                Row {
                    id: resetRow
                    anchors.centerIn: parent
                    spacing: 6

                    Icon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "rotate-ccw"
                        size: 14
                        color: panel.theme.text
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Tout réinitialiser"
                        color: panel.theme.text
                        font.pixelSize: 12
                    }
                }
                MouseArea {
                    id: resetArea
                    anchors.fill: parent
                    enabled: panel.modified
                    hoverEnabled: true
                    onClicked: panel.rows.forEach(r => panel.image[r.key] = r.neutral)
                }
            }
        }
    }
}
