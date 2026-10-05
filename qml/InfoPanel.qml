import QtQuick

// Informations sur le fichier en cours (Tab), à la manière de l'écran d'infos de PotPlayer :
// fichier, vidéo, audio, lecture. Relu chaque seconde tant qu'il est affiché.
Rectangle {
    id: panel

    property var video
    property var utils
    property var theme
    property string url: ""

    // Sections affichées : [{ title, rows: [[nom, valeur]] }]
    property var sections: []

    function read(name) {
        const value = video.getProperty(name)
        return value === undefined || value === null ? "" : value
    }

    function refresh() {
        const width = read("video-params/w"), height = read("video-params/h")
        const fps = read("estimated-vf-fps") || read("container-fps")
        const hwdec = read("hwdec-current")
        const channels = read("audio-params/channel-count")
        const sampleRate = read("audio-params/samplerate")
        const dropped = (read("frame-drop-count") || 0) + (read("decoder-frame-drop-count") || 0)
        const cache = read("demuxer-cache-duration")

        sections = [
            { title: "Fichier", rows: [
                ["Nom", utils.fileName(url)],
                ["Format", String(read("file-format")).split(",")[0].toUpperCase()],
                ["Taille", utils.formatBytes(read("file-size") || 0)],
                ["Durée", utils.formatClock(video.duration)]
            ] },
            { title: "Vidéo", rows: [
                ["Codec", utils.codecName(read("video-codec-name") || read("video-format"))],
                ["Résolution", width && height ? width + " × " + height : ""],
                ["Images/s", fps ? Number(fps).toFixed(3).replace(/\.?0+$/, "").replace(".", ",") : ""],
                ["Débit", utils.formatBitrate(read("video-bitrate") || 0)],
                ["Pixels", read("video-params/pixelformat")],
                ["Décodage", hwdec && hwdec !== "no" ? "Matériel (" + hwdec + ")" : "Logiciel"]
            ] },
            { title: "Audio", rows: [
                ["Codec", utils.codecName(read("audio-codec-name"))],
                ["Canaux", channels ? String(channels) : ""],
                ["Fréquence", sampleRate ? (sampleRate / 1000).toString().replace(".", ",") + " kHz" : ""],
                ["Débit", utils.formatBitrate(read("audio-bitrate") || 0)]
            ] },
            { title: "Lecture", rows: [
                ["Images perdues", String(dropped)],
                ["Cache", cache ? Number(cache).toFixed(1).replace(".", ",") + " s" : ""],
                ["Vitesse", "×" + video.speed.toFixed(2).replace(/\.?0+$/, "").replace(".", ",")]
            ] }
        ].map(s => ({ title: s.title, rows: s.rows.filter(r => r[1] !== "") }))
         .filter(s => s.rows.length > 0)
    }

    width: 340
    height: content.implicitHeight + 24
    radius: 10
    color: Qt.rgba(0, 0, 0, 0.72)
    border.color: Qt.rgba(1, 1, 1, 0.12)

    onVisibleChanged: if (visible) refresh()

    Timer {
        interval: 1000
        repeat: true
        running: panel.visible
        onTriggered: panel.refresh()
    }

    Column {
        id: content
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 12 }
        spacing: 10

        Repeater {
            model: panel.sections
            delegate: Column {
                required property var modelData
                width: content.width
                spacing: 3

                Text {
                    text: modelData.title
                    color: panel.theme.accent
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                }
                Repeater {
                    model: modelData.rows
                    delegate: Row {
                        required property var modelData
                        spacing: 10
                        Text {
                            width: 110
                            text: modelData[0]
                            color: Qt.rgba(1, 1, 1, 0.6)
                            font.pixelSize: 12
                        }
                        Text {
                            width: content.width - 120
                            text: modelData[1]
                            color: panel.theme.onVideo
                            font.pixelSize: 12
                            font.family: "monospace"
                            elide: Text.ElideMiddle
                        }
                    }
                }
            }
        }
    }
}
