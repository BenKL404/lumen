import QtQuick

// Barre de contrôle : fixe sous l'image en mode fenêtré, flottante en plein écran
Rectangle {
    id: bar

    property var video
    property var utils
    property var theme
    property bool floating: false
    property bool hasPrevious: false
    property bool hasNext: false
    property int repeatMode: 0 // 0 désactivé, 1 le fichier, 2 la playlist
    property bool shuffle: false
    property var bookmarks: []
    property bool imagePanelOpen: false
    property string mediaUrl: ""
    // Clic sur la durée : afficher le temps restant (comme PotPlayer)
    property bool showRemaining: false
    readonly property bool hovered: hover.hovered

    // Fenêtre étroite : les boutons secondaires se masquent par paliers (toujours accessibles
    // par le clic droit). Seuils calculés sur la largeur des boutons (36 px + 2 d'espacement).
    // 0 : tout ; 1 : sans capture ni vitesse ; 2 : sans aléatoire ni répétition ;
    // 3 : sans audio, sous-titres ni réglages d'image ; 4 : sans stop ni ouvrir ; 5 : sans durée totale
    readonly property int compact: {
        const w = width - 24
        return w >= 760 ? 0 : w >= 684 ? 1 : w >= 608 ? 2 : w >= 494 ? 3 : w >= 418 ? 4 : 5
    }

    // Infos techniques ; une pochette d'album n'est pas une vraie piste vidéo
    readonly property var videoTrack: video.tracks.find(t => t.type === "video" && t.selected)
    readonly property bool realVideo: videoTrack !== undefined && !videoTrack.albumart && !videoTrack.image
    readonly property string infoText: {
        const i = video.info
        return utils.mediaInfo(realVideo ? i["video-format"] || "" : "",
                               realVideo ? i["width"] || 0 : 0,
                               realVideo ? i["height"] || 0 : 0,
                               i["audio-codec-name"] || "",
                               i["audio-params/samplerate"] || 0,
                               i["audio-bitrate"] || 0)
    }

    signal openRequested()
    signal stopRequested()
    signal fullscreenRequested()
    signal audioMenuRequested()
    signal subtitleMenuRequested()
    signal previousRequested()
    signal nextRequested()
    signal playlistRequested()
    signal repeatRequested()
    signal shuffleRequested()
    signal abLoopClearRequested()
    signal imageSettingsRequested()

    height: 84
    radius: floating ? theme.radius : 0
    color: floating ? theme.surface : theme.chrome
    border.color: floating ? theme.border : "transparent"

    HoverHandler { id: hover }

    // Liseré supérieur en mode fixe
    Rectangle {
        visible: !bar.floating
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 1
        color: bar.theme.border
    }

    Column {
        anchors { fill: parent; leftMargin: 12; rightMargin: 12; topMargin: 8; bottomMargin: 6 }
        spacing: 2

        // ---------------------------------------- Progression et volume
        Item {
            width: parent.width
            height: 26

            SeekBar {
                anchors { left: parent.left; right: volumeGroup.left; rightMargin: 18; verticalCenter: parent.verticalCenter }
                theme: bar.theme
                value: bar.video.position
                to: bar.video.duration
                enabled: bar.video.hasMedia
                tooltipFormatter: (seconds) => bar.utils.formatClock(seconds)
                chapters: bar.video.chapters
                // Miniatures par paliers (environ 200 par vidéo) : moins de calculs, cache efficace
                previewSource: bar.realVideo && bar.mediaUrl !== ""
                    ? (seconds) => {
                        const step = Math.max(1, bar.video.duration / 200)
                        return bar.utils.thumbnailUrl(bar.mediaUrl, Math.round(seconds / step) * step)
                    }
                    : null
                bookmarks: bar.bookmarks
                loopA: bar.video.abLoopA
                loopB: bar.video.abLoopB
                onMoved: (seconds) => bar.video.seekAbsolute(seconds)
            }

            Row {
                id: volumeGroup
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                spacing: 6

                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    implicitHeight: 26
                    theme: bar.theme
                    icon: bar.video.muted || bar.video.volume <= 0 ? "volume-x" : "volume"
                    iconSize: 16
                    onClicked: bar.video.muted = !bar.video.muted
                }
                SeekBar {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 96
                    theme: bar.theme
                    value: bar.video.volume
                    to: 130
                    live: true
                    opacity: bar.video.muted ? 0.4 : 1
                    onMoved: (v) => bar.video.volume = v
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 34
                    text: Math.round(bar.video.volume)
                    // Au-delà de 100 % : amplification, signalée en couleur d'accent
                    color: bar.video.volume > 100 ? bar.theme.accent : bar.theme.muted
                    font.pixelSize: 12
                    font.family: "monospace"
                }
            }
        }

        // ---------------------------------------- Boutons
        Item {
            width: parent.width
            height: 42

            Row {
                id: leftGroup
                anchors { left: parent.left; verticalCenter: parent.verticalCenter }
                spacing: 2

                IconButton {
                    theme: bar.theme
                    icon: bar.video.paused || !bar.video.hasMedia ? "play" : "pause"
                    filled: true
                    iconSize: 20
                    onClicked: bar.video.hasMedia ? bar.video.togglePause() : bar.openRequested()
                }
                IconButton { theme: bar.theme; icon: "stop"; filled: true; iconSize: 16; visible: bar.compact < 4; enabled: bar.video.hasMedia; onClicked: bar.stopRequested() }
                IconButton { theme: bar.theme; icon: "skip-back"; filled: true; enabled: bar.hasPrevious; onClicked: bar.previousRequested() }
                IconButton { theme: bar.theme; icon: "skip-forward"; filled: true; enabled: bar.hasNext; onClicked: bar.nextRequested() }
                IconButton { theme: bar.theme; icon: "folder-open"; visible: bar.compact < 4; onClicked: bar.openRequested() }

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    leftPadding: 14

                    Text {
                        text: bar.utils.formatClock(bar.video.position)
                        color: bar.theme.text
                        font.pixelSize: 14
                        font.family: "monospace"
                    }
                    Text {
                        visible: bar.compact < 5
                        text: "  /  "
                        color: bar.theme.muted
                        font.pixelSize: 14
                        font.family: "monospace"
                    }
                    Text {
                        visible: bar.compact < 5
                        text: bar.showRemaining
                              ? "-" + bar.utils.formatClock(bar.video.duration - bar.video.position)
                              : bar.utils.formatClock(bar.video.duration)
                        color: durationArea.containsMouse ? bar.theme.text : bar.theme.muted
                        font.pixelSize: 14
                        font.family: "monospace"

                        MouseArea {
                            id: durationArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: bar.showRemaining = !bar.showRemaining
                        }
                    }

                    Item { width: 10; height: 1; visible: bar.video.abLoopA >= 0 }

                    // Boucle A-B active : « A » puis « A-B » ; un clic la retire
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: bar.video.abLoopA >= 0
                        width: abText.implicitWidth + 12
                        height: 20
                        radius: 4
                        color: abArea.containsMouse ? Qt.rgba(1, 1, 1, 0.08) : "transparent"
                        border.color: bar.theme.accent

                        Text {
                            id: abText
                            anchors.centerIn: parent
                            text: bar.video.abLoopB >= 0 ? "A-B" : "A"
                            color: bar.theme.accent
                            font.pixelSize: 11
                            font.weight: Font.DemiBold
                        }
                        MouseArea {
                            id: abArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: bar.abLoopClearRequested()
                        }
                    }
                }
            }

            Text {
                anchors { left: leftGroup.right; leftMargin: 18; right: rightGroup.left; rightMargin: 12; verticalCenter: parent.verticalCenter }
                text: bar.infoText
                // Masquées plutôt que tronquées quand la place manque
                visible: implicitWidth <= width
                color: bar.theme.muted
                font.pixelSize: 12
            }

            Row {
                id: rightGroup
                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                spacing: 2

                IconButton { theme: bar.theme; icon: "audio-lines"; visible: bar.compact < 3; onClicked: bar.audioMenuRequested() }
                IconButton { theme: bar.theme; icon: "captions"; visible: bar.compact < 3; onClicked: bar.subtitleMenuRequested() }
                IconButton {
                    theme: bar.theme
                    visible: bar.compact < 1
                    glyph: "×" + bar.video.speed.toFixed(bar.video.speed % 1 === 0 ? 0 : 2)
                    onClicked: bar.video.speed = bar.video.speed >= 2 ? 1.0 : bar.video.speed + 0.25
                }
                IconButton { theme: bar.theme; icon: "camera"; visible: bar.compact < 1; enabled: bar.video.hasMedia; onClicked: bar.video.screenshot() }
                IconButton { theme: bar.theme; icon: "sun"; visible: bar.compact < 3; active: bar.imagePanelOpen; onClicked: bar.imageSettingsRequested() }
                IconButton { theme: bar.theme; icon: "shuffle"; visible: bar.compact < 2; active: bar.shuffle; onClicked: bar.shuffleRequested() }
                IconButton {
                    theme: bar.theme
                    visible: bar.compact < 2
                    icon: bar.repeatMode === 1 ? "repeat-1" : "repeat"
                    active: bar.repeatMode !== 0
                    onClicked: bar.repeatRequested()
                }
                IconButton { theme: bar.theme; icon: "list"; onClicked: bar.playlistRequested() }
                IconButton { theme: bar.theme; icon: bar.floating ? "minimize" : "maximize"; onClicked: bar.fullscreenRequested() }
            }
        }
    }
}
