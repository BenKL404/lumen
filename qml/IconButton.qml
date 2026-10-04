import QtQuick

Item {
    id: button

    property var theme
    property string glyph
    property bool emphasized: false

    signal clicked()

    implicitWidth: Math.max(38, label.implicitWidth + 18)
    implicitHeight: 38
    opacity: enabled ? 1 : 0.35

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: button.emphasized ? button.theme.accent
             : area.containsMouse ? button.theme.surfaceHover : "transparent"
        scale: area.pressed ? 0.92 : 1
        Behavior on scale { NumberAnimation { duration: 90 } }
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Text {
        id: label
        anchors.centerIn: parent
        text: button.glyph
        color: button.emphasized ? "#1A1206" : button.theme.text
        font.pixelSize: 15
        font.weight: Font.DemiBold
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: button.clicked()
    }
}
