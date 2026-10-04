import QtQuick
import QtQuick.Window

// Barre de titre intégrée (fenêtre sans bordure) : menu, format, titre, épingle et boutons de fenêtre
Rectangle {
    id: bar

    property var window
    property var theme
    property string format: ""
    property string title: "Lumen"
    property bool pinned: false

    signal menuRequested(Item anchor)
    signal pinToggled()
    signal maximizeToggled()

    height: 36
    color: theme.chrome

    Rectangle {
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 1
        color: bar.theme.border
    }

    // Glisser pour déplacer la fenêtre, double-clic pour l'agrandir
    DragHandler {
        target: null
        onActiveChanged: if (active) bar.window.startSystemMove()
    }
    TapHandler {
        onDoubleTapped: bar.maximizeToggled()
    }

    Row {
        id: leftGroup
        anchors { left: parent.left; leftMargin: 6; verticalCenter: parent.verticalCenter }
        spacing: 8

        // Menu principal « Lumen ▾ »
        Rectangle {
            id: menuButton
            width: menuLabel.implicitWidth + 20
            height: 26
            radius: 6
            color: menuArea.containsMouse ? bar.theme.surfaceHover : "transparent"

            Text {
                id: menuLabel
                anchors.centerIn: parent
                text: "Lumen  ▾"
                color: bar.theme.accent
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            MouseArea {
                id: menuArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: bar.menuRequested(menuButton)
            }
        }

        // Badge du format (MKV, MP4, MP3…)
        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.format !== ""
            width: formatLabel.implicitWidth + 12
            height: 18
            radius: 4
            color: "transparent"
            border.color: bar.theme.border

            Text {
                id: formatLabel
                anchors.centerIn: parent
                text: bar.format
                color: bar.theme.muted
                font.pixelSize: 10
                font.weight: Font.DemiBold
                font.letterSpacing: 0.5
            }
        }
    }

    Text {
        anchors { left: leftGroup.right; leftMargin: 12; right: windowButtons.left; rightMargin: 12; verticalCenter: parent.verticalCenter }
        text: bar.title
        color: bar.theme.text
        font.pixelSize: 13
        elide: Text.ElideMiddle
    }

    component WindowButton: Rectangle {
        id: wb

        property var theme
        property string glyph
        property bool active: false
        property color hoverColor: theme.surfaceHover
        signal clicked()

        width: 42
        height: 36
        color: wbArea.containsMouse ? hoverColor : "transparent"

        Text {
            anchors.centerIn: parent
            text: wb.glyph
            color: wb.active ? wb.theme.accent : wb.theme.text
            font.pixelSize: 13
        }
        MouseArea {
            id: wbArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: wb.clicked()
        }
    }

    Row {
        id: windowButtons
        anchors { right: parent.right; top: parent.top }

        WindowButton { theme: bar.theme; glyph: "⚲"; active: bar.pinned; onClicked: bar.pinToggled() }
        WindowButton { theme: bar.theme; glyph: "─"; onClicked: bar.window.showMinimized() }
        WindowButton {
            theme: bar.theme
            glyph: bar.window.visibility === Window.Maximized ? "❐" : "☐"
            onClicked: bar.maximizeToggled()
        }
        WindowButton { theme: bar.theme; glyph: "✕"; hoverColor: "#C42B1C"; onClicked: bar.window.close() }
    }
}
