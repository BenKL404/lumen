import QtQuick

// Fenêtre « Raccourcis clavier » : toutes les actions et leurs touches (F1).
// Les touches se modifient dans settings.toml ; « Recharger » les applique aussitôt.
Rectangle {
    id: panel

    property var theme
    property var settings
    // Touche en clair (« Espace », « Ctrl+O »…), fournie par Main.qml
    property var formatKey: (key) => key

    signal closeRequested()
    signal editRequested()
    signal reloadRequested()

    readonly property string warnings: { settings.shortcutsVersion; return settings.shortcutWarnings() }

    width: 540
    radius: theme.radius
    color: theme.menu
    border.color: theme.border

    // Absorbe les clics : ne pas mettre en pause en cliquant dans la fenêtre
    MouseArea { anchors.fill: parent }

    // ------------------------------------------------------------ En-tête
    Item {
        id: header
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 52

        Text {
            anchors { left: parent.left; leftMargin: 20; verticalCenter: parent.verticalCenter }
            text: "Raccourcis clavier"
            color: panel.theme.text
            font.pixelSize: 16
            font.weight: Font.DemiBold
        }
        IconButton {
            anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
            implicitHeight: 32
            theme: panel.theme
            icon: "x"
            iconSize: 15
            onClicked: panel.closeRequested()
        }
        Rectangle {
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
            height: 1
            color: panel.theme.border
        }
    }

    // Conflits ou actions inconnues dans le fichier
    Text {
        id: warningText
        anchors { left: parent.left; right: parent.right; top: header.bottom; margins: 16 }
        visible: panel.warnings !== ""
        text: panel.warnings
        color: panel.theme.accent
        font.pixelSize: 12
        wrapMode: Text.WordWrap
    }

    // ------------------------------------------------------------ Liste
    ListView {
        id: list
        anchors {
            left: parent.left; right: parent.right
            top: warningText.visible ? warningText.bottom : header.bottom
            bottom: footer.top
            margins: 8
        }
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        model: panel.settings.shortcutActions()

        delegate: Rectangle {
            id: row

            required property string modelData
            required property int index
            readonly property var keys: { panel.settings.shortcutsVersion; return panel.settings.keys(modelData) }

            width: ListView.view.width
            height: 34
            radius: 6
            color: index % 2 === 0 ? panel.theme.subtle : "transparent"

            Text {
                anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                text: panel.settings.actionLabel(row.modelData)
                color: panel.theme.text
                font.pixelSize: 13
            }

            // Une « touche » dessinée par raccourci ; tiret si l'action n'en a pas
            Row {
                anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
                spacing: 6

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: row.keys.length === 0
                    text: "—"
                    color: panel.theme.muted
                    font.pixelSize: 13
                }
                Repeater {
                    model: row.keys
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
                            text: panel.formatKey(parent.modelData)
                            color: panel.theme.text
                            font.pixelSize: 12
                            font.weight: Font.Medium
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ Pied
    Item {
        id: footer
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 56

        Rectangle {
            anchors { left: parent.left; right: parent.right; top: parent.top }
            height: 1
            color: panel.theme.border
        }
        Text {
            anchors { left: parent.left; leftMargin: 20; right: buttons.left; rightMargin: 12; verticalCenter: parent.verticalCenter }
            text: "Section [raccourcis] de settings.toml"
            color: panel.theme.muted
            font.pixelSize: 12
            elide: Text.ElideRight
        }

        component FooterButton: Rectangle {
            id: fb
            property var theme
            property string label
            property string icon
            signal clicked()

            width: fbRow.implicitWidth + 22
            height: 30
            radius: 6
            color: fbArea.containsMouse ? theme.surfaceHover : panel.theme.subtle

            Row {
                id: fbRow
                anchors.centerIn: parent
                spacing: 6
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: fb.icon
                    size: 14
                    color: fb.theme.text
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: fb.label
                    color: fb.theme.text
                    font.pixelSize: 12
                }
            }
            MouseArea {
                id: fbArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: fb.clicked()
            }
        }

        Row {
            id: buttons
            anchors { right: parent.right; rightMargin: 14; verticalCenter: parent.verticalCenter }
            spacing: 8

            FooterButton { theme: panel.theme; icon: "pencil"; label: "Modifier…"; onClicked: panel.editRequested() }
            FooterButton { theme: panel.theme; icon: "refresh"; label: "Recharger"; onClicked: panel.reloadRequested() }
        }
    }
}
