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
    property var tooltipFormatter: null

    signal moved(real value)

    readonly property real shownValue: area.pressed ? dragValue : value
    property real dragValue: 0

    implicitHeight: 20
    opacity: enabled ? 1 : 0.4

    function valueAt(x) {
        return Math.max(0, Math.min(1, x / width)) * to
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
            width: slider.to > 0 ? parent.width * Math.min(1, slider.shownValue / slider.to) : 0
            height: parent.height
            radius: parent.radius
            color: slider.theme.accent
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

    // Info-bulle temporelle au survol
    Rectangle {
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
            text: slider.tooltipFormatter ? slider.tooltipFormatter(slider.valueAt(area.mouseX)) : ""
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
