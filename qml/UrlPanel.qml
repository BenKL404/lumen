import QtQuick

// « Ouvrir une URL » (Ctrl+U) : lien YouTube ou de tout site pris en charge par yt-dlp,
// ou adresse directe d'un flux vidéo.
Rectangle {
    id: panel

    property var theme

    signal accepted(string url)
    signal closeRequested()

    function open(initial) {
        field.text = initial || ""
        visible = true
        field.forceActiveFocus()
        field.selectAll()
    }

    function submit() {
        const url = field.text.trim()
        if (url !== "")
            accepted(url)
    }

    width: 560
    height: 150
    radius: theme.radius
    color: theme.menu
    border.color: theme.border

    // Absorbe les clics : ne pas mettre en pause en cliquant dans la fenêtre
    MouseArea { anchors.fill: parent }

    Text {
        id: title
        anchors { left: parent.left; top: parent.top; leftMargin: 20; topMargin: 18 }
        text: "Ouvrir une vidéo en ligne"
        color: panel.theme.text
        font.pixelSize: 15
        font.weight: Font.DemiBold
    }
    IconButton {
        anchors { right: parent.right; rightMargin: 10; verticalCenter: title.verticalCenter }
        implicitHeight: 32
        theme: panel.theme
        icon: "x"
        iconSize: 15
        onClicked: panel.closeRequested()
    }

    Rectangle {
        id: box
        anchors { left: parent.left; right: openButton.left; top: title.bottom; leftMargin: 20; rightMargin: 10; topMargin: 16 }
        height: 36
        radius: 6
        color: panel.theme.field
        border.color: field.activeFocus ? panel.theme.accent : panel.theme.border

        TextInput {
            id: field
            anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
            verticalAlignment: TextInput.AlignVCenter
            color: panel.theme.text
            selectionColor: panel.theme.accent
            font.pixelSize: 13
            clip: true
            onAccepted: panel.submit()
            Keys.onEscapePressed: panel.closeRequested()
        }
        Text {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
            visible: field.text === ""
            text: "https://www.youtube.com/watch?v=…"
            color: panel.theme.muted
            font.pixelSize: 13
        }
    }

    Rectangle {
        id: openButton
        anchors { right: parent.right; rightMargin: 20; verticalCenter: box.verticalCenter }
        width: openLabel.implicitWidth + 28
        height: 36
        radius: 6
        color: openArea.containsMouse ? Qt.lighter(panel.theme.accent, 1.1) : panel.theme.accent
        opacity: field.text.trim() !== "" ? 1 : 0.5

        Text {
            id: openLabel
            anchors.centerIn: parent
            text: "Ouvrir"
            color: panel.theme.onAccent
            font.pixelSize: 13
            font.weight: Font.DemiBold
        }
        MouseArea {
            id: openArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: panel.submit()
        }
    }

    Text {
        anchors { left: parent.left; right: parent.right; top: box.bottom; leftMargin: 20; rightMargin: 20; topMargin: 12 }
        text: "YouTube, Vimeo, Dailymotion… et la plupart des sites vidéo (via yt-dlp)"
        color: panel.theme.muted
        font.pixelSize: 12
        elide: Text.ElideRight
    }
}
