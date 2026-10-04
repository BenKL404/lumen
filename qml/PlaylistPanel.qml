import QtQuick

// Panneau latéral de la playlist, qui glisse depuis la droite
Rectangle {
    id: panel

    property var playlist
    property var utils
    property var theme
    property bool open: false

    signal activated(int index)

    color: theme.surface
    border.color: Qt.rgba(1, 1, 1, 0.06)

    // Absorbe les clics : ne pas mettre en pause en cliquant dans le panneau
    MouseArea { anchors.fill: parent }

    onOpenChanged: if (open) list.positionViewAtIndex(playlist.current, ListView.Center)

    Item {
        id: header
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 64

        Text {
            anchors { left: parent.left; leftMargin: 22; verticalCenter: parent.verticalCenter }
            text: "Playlist"
            color: panel.theme.text
            font.pixelSize: 17
            font.weight: Font.Medium
        }
        Text {
            anchors { right: closeButton.left; rightMargin: 8; verticalCenter: parent.verticalCenter }
            text: (panel.playlist.current + 1) + " / " + panel.playlist.items.length
            color: panel.theme.muted
            font.pixelSize: 13
            font.family: "monospace"
        }
        IconButton {
            id: closeButton
            anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            theme: panel.theme
            glyph: "✕"
            onClicked: panel.open = false
        }
    }

    ListView {
        id: list
        anchors { left: parent.left; right: parent.right; top: header.bottom; bottom: parent.bottom; margins: 10 }
        clip: true
        spacing: 2
        model: panel.playlist.items

        delegate: Rectangle {
            id: row

            required property string modelData
            required property int index
            readonly property bool isCurrent: index === panel.playlist.current

            width: ListView.view.width
            height: 44
            radius: 8
            color: isCurrent ? Qt.rgba(panel.theme.accent.r, panel.theme.accent.g, panel.theme.accent.b, 0.12)
                 : rowArea.containsMouse ? panel.theme.surfaceHover : "transparent"

            Text {
                id: number
                anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
                width: 26
                text: row.index + 1
                color: row.isCurrent ? panel.theme.accent : panel.theme.muted
                font.pixelSize: 13
                font.family: "monospace"
            }
            Text {
                anchors { left: number.right; leftMargin: 8; right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
                text: panel.utils.fileName(row.modelData)
                color: row.isCurrent ? panel.theme.accent : panel.theme.text
                font.pixelSize: 14
                font.weight: row.isCurrent ? Font.Medium : Font.Normal
                elide: Text.ElideMiddle
            }
            MouseArea {
                id: rowArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.activated(row.index)
            }
        }
    }
}
