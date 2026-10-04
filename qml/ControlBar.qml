import QtQuick

Rectangle {
    id: bar

    property var video
    property var utils
    property var theme
    readonly property bool hovered: hover.hovered

    signal openRequested()
    signal fullscreenRequested()

    height: 96
    radius: theme.radius
    color: theme.surface
    border.color: Qt.rgba(1, 1, 1, 0.06)

    HoverHandler { id: hover }

    Column {
        anchors { fill: parent; leftMargin: 18; rightMargin: 18; topMargin: 14; bottomMargin: 10 }
        spacing: 8

        // Barre de progression
        SeekBar {
            width: parent.width
            theme: bar.theme
            value: bar.video.position
            to: bar.video.duration
            enabled: bar.video.hasMedia
            tooltipFormatter: (seconds) => bar.utils.formatTime(seconds)
            onMoved: (seconds) => bar.video.seekAbsolute(seconds)
        }

        Item {
            width: parent.width
            height: 40

            // Contrôles de lecture (gauche)
            Row {
                anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                spacing: 4

                IconButton {
                    theme: bar.theme
                    glyph: bar.video.paused || !bar.video.hasMedia ? "▶" : "❚❚"
                    emphasized: true
                    onClicked: bar.video.hasMedia ? bar.video.togglePause() : bar.openRequested()
                }
                IconButton { theme: bar.theme; glyph: "−10"; onClicked: bar.video.seekRelative(-10) }
                IconButton { theme: bar.theme; glyph: "+10"; onClicked: bar.video.seekRelative(10) }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    leftPadding: 10
                    text: bar.utils.formatTime(bar.video.position) + "  /  " + bar.utils.formatTime(bar.video.duration)
                    color: bar.theme.text
                    font.pixelSize: 14
                    font.family: "monospace"
                }
            }

            // Réglages (droite)
            Row {
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                spacing: 4

                IconButton {
                    theme: bar.theme
                    glyph: "×" + bar.video.speed.toFixed(bar.video.speed % 1 === 0 ? 0 : 2)
                    onClicked: bar.video.speed = bar.video.speed >= 2 ? 1.0 : bar.video.speed + 0.25
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Vol"
                    color: bar.theme.muted
                    font.pixelSize: 13
                    rightPadding: 6
                }
                SeekBar {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 110
                    theme: bar.theme
                    value: bar.video.volume
                    to: 130
                    live: true
                    onMoved: (v) => bar.video.volume = v
                }

                IconButton { theme: bar.theme; glyph: "◉"; onClicked: bar.video.screenshot() }
                IconButton { theme: bar.theme; glyph: "⏏"; onClicked: bar.openRequested() }
                IconButton { theme: bar.theme; glyph: "⤢"; onClicked: bar.fullscreenRequested() }
            }
        }
    }
}
