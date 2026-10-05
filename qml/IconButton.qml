import QtQuick

// Bouton rond : une icône vectorielle (`icon`) ou, à défaut, un court texte (`glyph`, ex. « ×1 »)
Item {
    id: button

    property var theme
    property string icon
    property string glyph
    property bool filled: false
    property real iconSize: 18
    property bool emphasized: false
    // Option activée (répétition, aléatoire…) : icône en couleur d'accent
    property bool active: false

    readonly property color contentColor: emphasized ? theme.onAccent : active ? theme.accent : theme.text

    signal clicked()

    implicitWidth: icon !== "" ? 36 : Math.max(36, label.implicitWidth + 16)
    implicitHeight: 36
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

    Icon {
        anchors.centerIn: parent
        visible: button.icon !== ""
        name: button.icon
        size: button.iconSize
        filled: button.filled
        color: button.contentColor
        scale: area.pressed ? 0.92 : 1
    }

    Text {
        id: label
        anchors.centerIn: parent
        visible: button.icon === ""
        text: button.glyph
        color: button.contentColor
        font.pixelSize: 13
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
