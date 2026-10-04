import QtQuick
import QtQuick.Window
import QtQuick.Dialogs
import com.lumen.video
import com.lumen.player

Window {
    id: root

    width: 1280
    height: 720
    minimumWidth: 640
    minimumHeight: 360
    visible: true
    color: theme.background
    title: video.hasMedia ? video.mediaTitle + " — Lumen" : "Lumen"

    // Design tokens : un seul endroit pour toute l'identité visuelle
    readonly property QtObject theme: QtObject {
        readonly property color background: "#0B0D10"
        readonly property color surface: Qt.rgba(0.07, 0.08, 0.10, 0.80)
        readonly property color surfaceHover: Qt.rgba(1, 1, 1, 0.08)
        readonly property color text: "#ECE8E1"
        readonly property color muted: "#8B9099"
        readonly property color track: Qt.rgba(1, 1, 1, 0.16)
        readonly property color accent: "#F2A541"
        readonly property int radius: 16
        readonly property int animation: 180
    }

    property bool controlsVisible: true

    // Média en cours (clé de l'historique) ; vide tant que le fichier n'est pas chargé,
    // pour ne jamais enregistrer la position d'un fichier sous le nom d'un autre.
    property string currentUrl: ""
    property string pendingUrl: ""

    function wake() {
        controlsVisible = true
        hideTimer.restart()
    }

    function toggleFullScreen() {
        root.visibility = root.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen
    }

    function osd(message) {
        osdText.text = message
        osdBox.opacity = 1
        osdTimer.restart()
    }

    function openUrl(url) {
        saveProgress()
        currentUrl = ""
        pendingUrl = url.toString()
        resumeBox.hide()
        video.loadFile(url)
        osd(utils.fileName(pendingUrl))
    }

    function selectedTrack(type) {
        return video.tracks.find(t => t.type === type && t.selected)
    }

    function saveProgress() {
        if (currentUrl === "")
            return
        const audio = selectedTrack("audio")
        const sub = selectedTrack("sub")
        // Les sous-titres externes sont mémorisés par chemin : leur id change à chaque ouverture
        const subFile = sub && sub.external ? sub["external-filename"] || "" : ""
        const subId = sub ? (subFile !== "" ? -1 : sub.id) : (video.tracks.length > 0 ? 0 : -1)
        history.remember(currentUrl, video.position, video.duration, video.speed,
                         audio ? audio.id : -1, subId, video.subDelay, subFile)
    }

    function shiftSubDelay(step) {
        const delay = Math.round((video.subDelay + step) * 10) / 10
        video.subDelay = delay
        osd("Décalage sous-titres " + (delay > 0 ? "+" : "") + delay.toFixed(1) + " s")
    }

    function addSubtitle(url) {
        // « cached » : resélectionne le fichier s'il est déjà chargé au lieu d'un doublon
        video.command(["sub-add", utils.localPath(url.toString()), "cached"])
        osd("Sous-titres : " + utils.fileName(url.toString()))
    }

    function toggleTrackMenu(kind) {
        if (trackMenu.visible && trackMenu.kind === kind) {
            trackMenu.visible = false
        } else {
            trackMenu.kind = kind
            trackMenu.visible = true
        }
    }

    // Reprise automatique à la position enregistrée
    function resume() {
        const found = history.load(currentUrl)
        // Le décalage est un réglage global de mpv : le remettre à zéro pour un nouveau fichier
        video.subDelay = history.subDelay
        if (!found)
            return

        if (history.audioId > 0)
            video.command(["set", "aid", String(history.audioId)])
        if (history.subFile !== "")
            video.command(["sub-add", history.subFile, "cached"])
        else if (history.subId > 0)
            video.command(["set", "sid", String(history.subId)])
        else if (history.subId === 0)
            video.command(["set", "sid", "no"])

        video.speed = history.speed
        if (history.position > 0) {
            video.seekAbsolute(history.position)
            resumeBox.show(history.position)
        }
    }

    onClosing: saveProgress()

    Utils { id: utils }
    History { id: history }

    // ---------------------------------------------------------------- Vidéo
    MpvVideo {
        id: video
        anchors.fill: parent
        onPausedChanged: {
            root.wake()
            if (paused)
                root.saveProgress()
        }
        onFileLoaded: {
            root.currentUrl = root.pendingUrl
            saveTimer.restart()
            root.resume()
        }
    }

    // Sauvegarde régulière : la position survit à un plantage ou à une coupure
    Timer {
        id: saveTimer
        interval: 5000
        repeat: true
        running: video.hasMedia && !video.paused
        onTriggered: root.saveProgress()
    }

    // ---------------------------------------------------------- État vide
    Column {
        anchors.centerIn: parent
        spacing: 12
        visible: !video.hasMedia

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Lumen"
            color: theme.text
            font.pixelSize: 46
            font.weight: Font.Light
            font.letterSpacing: 2
        }
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Glissez une vidéo ici, ou appuyez sur O pour ouvrir un fichier"
            color: theme.muted
            font.pixelSize: 15
        }
    }

    // ------------------------------------------------- Interactions souris
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.controlsVisible ? Qt.ArrowCursor : Qt.BlankCursor

        onPositionChanged: root.wake()
        onClicked: clickTimer.restart()
        onDoubleClicked: {
            clickTimer.stop()
            root.toggleFullScreen()
        }
        onWheel: (wheel) => {
            const step = wheel.angleDelta.y > 0 ? 5 : -5
            video.volume = video.volume + step
            root.osd("Volume " + Math.round(Math.max(0, Math.min(130, video.volume + step))) + " %")
        }

        // Distingue clic simple (pause) et double-clic (plein écran)
        Timer {
            id: clickTimer
            interval: 220
            onTriggered: if (video.hasMedia) video.togglePause()
        }
    }

    Timer {
        id: hideTimer
        interval: 2500
        onTriggered: {
            if (video.hasMedia && !video.paused && !controls.hovered && !trackMenu.visible)
                root.controlsVisible = false
        }
    }

    // ------------------------------------------------------- Barre du haut
    Rectangle {
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 90
        visible: video.hasMedia
        opacity: root.controlsVisible ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: theme.animation } }
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.rgba(0, 0, 0, 0.65) }
            GradientStop { position: 1.0; color: "transparent" }
        }

        Text {
            anchors { left: parent.left; top: parent.top; margins: 22; right: parent.right }
            text: video.mediaTitle
            color: theme.text
            font.pixelSize: 17
            font.weight: Font.Medium
            elide: Text.ElideRight
        }
    }

    // ------------------------------------------------- Barre de contrôle
    ControlBar {
        id: controls
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: 20 }
        video: video
        utils: utils
        theme: root.theme
        opacity: root.controlsVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.animation } }

        onOpenRequested: fileDialog.open()
        onFullscreenRequested: root.toggleFullScreen()
        onAudioMenuRequested: root.toggleTrackMenu("audio")
        onSubtitleMenuRequested: root.toggleTrackMenu("sub")
    }

    // ------------------------------------------------ Menu des pistes
    // Un clic en dehors du menu le ferme (sans mettre la vidéo en pause)
    MouseArea {
        anchors.fill: parent
        enabled: trackMenu.visible
        onClicked: trackMenu.visible = false
    }

    TrackMenu {
        id: trackMenu
        anchors { right: controls.right; bottom: controls.top; bottomMargin: 10 }
        visible: false
        video: video
        utils: utils
        theme: root.theme
        onAddSubtitleRequested: {
            trackMenu.visible = false
            subtitleDialog.open()
        }
    }

    // ------------------------------------------------------------- OSD
    Rectangle {
        id: osdBox
        anchors { top: parent.top; right: parent.right; margins: 24 }
        width: osdText.implicitWidth + 32
        height: 40
        radius: 12
        color: theme.surface
        opacity: 0
        Behavior on opacity { NumberAnimation { duration: theme.animation } }

        Text {
            id: osdText
            anchors.centerIn: parent
            color: theme.text
            font.pixelSize: 14
        }
        Timer {
            id: osdTimer
            interval: 1200
            onTriggered: osdBox.opacity = 0
        }
    }

    // ------------------------------------------------ Reprise de lecture
    Rectangle {
        id: resumeBox
        anchors { top: osdBox.bottom; right: parent.right; topMargin: 8; rightMargin: 24 }
        width: resumeRow.implicitWidth + 32
        height: 40
        radius: 12
        color: theme.surface
        opacity: 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.animation } }

        function show(position) {
            resumeText.text = "Reprise à " + utils.formatTime(position)
            opacity = 1
            resumeTimer.restart()
        }
        function hide() {
            resumeTimer.stop()
            opacity = 0
        }

        // Absorbe les clics : pas de pause en cliquant sur le message
        MouseArea {
            id: resumeHover
            anchors.fill: parent
            hoverEnabled: true
        }

        Row {
            id: resumeRow
            anchors.centerIn: parent
            spacing: 14

            Text {
                id: resumeText
                anchors.verticalCenter: parent.verticalCenter
                color: theme.text
                font.pixelSize: 14
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "Recommencer"
                color: theme.accent
                font.pixelSize: 14
                font.weight: Font.Medium
                font.underline: restartArea.containsMouse

                MouseArea {
                    id: restartArea
                    anchors.fill: parent
                    anchors.margins: -6
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        history.forget(root.currentUrl)
                        video.seekAbsolute(0)
                        resumeBox.hide()
                        root.osd("Lecture depuis le début")
                    }
                }
            }
        }

        Timer {
            id: resumeTimer
            interval: 6000
            onTriggered: if (resumeHover.containsMouse) restart(); else resumeBox.opacity = 0
        }
    }

    // ------------------------------------------------ Glisser-déposer
    DropArea {
        anchors.fill: parent
        onDropped: (drop) => {
            if (!drop.hasUrls || drop.urls.length === 0)
                return
            const url = drop.urls[0]
            if (video.hasMedia && utils.isSubtitle(url.toString()))
                root.addSubtitle(url)
            else
                root.openUrl(url)
        }
    }

    FileDialog {
        id: fileDialog
        title: "Ouvrir une vidéo"
        nameFilters: [
            "Vidéos (*.mkv *.mp4 *.avi *.webm *.mov *.wmv *.flv *.ts *.m2ts *.mpg *.ogv)",
            "Audio (*.mp3 *.flac *.ogg *.opus *.wav *.m4a *.aac)",
            "Tous les fichiers (*)"
        ]
        onAccepted: root.openUrl(selectedFile)
    }

    FileDialog {
        id: subtitleDialog
        title: "Ajouter des sous-titres"
        // Ouvre le dossier de la vidéo, où se trouvent en général ses sous-titres
        currentFolder: root.currentUrl.substring(0, root.currentUrl.lastIndexOf("/"))
        nameFilters: [
            "Sous-titres (*.srt *.ass *.ssa *.vtt *.sub *.sup *.idx *.smi)",
            "Tous les fichiers (*)"
        ]
        onAccepted: root.addSubtitle(selectedFile)
    }

    // ------------------------------------------------ Raccourcis clavier
    Shortcut { sequence: "Space"; onActivated: video.togglePause() }
    Shortcut { sequence: "Left"; onActivated: { video.seekRelative(-5); root.osd("−5 s") } }
    Shortcut { sequence: "Right"; onActivated: { video.seekRelative(5); root.osd("+5 s") } }
    Shortcut { sequence: "Ctrl+Left"; onActivated: { video.seekRelative(-30); root.osd("−30 s") } }
    Shortcut { sequence: "Ctrl+Right"; onActivated: { video.seekRelative(30); root.osd("+30 s") } }
    Shortcut {
        sequence: "Up"
        onActivated: { video.volume = video.volume + 5; root.osd("Volume " + Math.round(Math.min(130, video.volume + 5)) + " %") }
    }
    Shortcut {
        sequence: "Down"
        onActivated: { video.volume = video.volume - 5; root.osd("Volume " + Math.round(Math.max(0, video.volume - 5)) + " %") }
    }
    Shortcut { sequence: "M"; onActivated: { video.command(["cycle", "mute"]); root.osd("Muet") } }
    Shortcut { sequence: "F"; onActivated: root.toggleFullScreen() }
    Shortcut { sequence: "Return"; onActivated: root.toggleFullScreen() }
    Shortcut {
        sequence: "Escape"
        onActivated: {
            if (trackMenu.visible)
                trackMenu.visible = false
            else if (root.visibility === Window.FullScreen)
                root.visibility = Window.Windowed
        }
    }
    Shortcut { sequence: "O"; onActivated: fileDialog.open() }
    Shortcut { sequence: "S"; onActivated: { video.screenshot(); root.osd("Capture enregistrée") } }
    Shortcut { sequence: "."; onActivated: video.frameStep(true) }
    Shortcut { sequence: ","; onActivated: video.frameStep(false) }
    Shortcut {
        sequence: "]"
        onActivated: { video.speed = video.speed + 0.1; root.osd("Vitesse ×" + Math.min(4, video.speed + 0.1).toFixed(1)) }
    }
    Shortcut {
        sequence: "["
        onActivated: { video.speed = video.speed - 0.1; root.osd("Vitesse ×" + Math.max(0.25, video.speed - 0.1).toFixed(1)) }
    }
    Shortcut { sequence: "Backspace"; onActivated: { video.speed = 1.0; root.osd("Vitesse normale") } }
    Shortcut { sequence: "J"; onActivated: { video.command(["cycle", "sub"]); root.osd("Sous-titres suivants") } }
    Shortcut { sequence: "Z"; onActivated: root.shiftSubDelay(-0.1) }
    Shortcut { sequence: "X"; onActivated: root.shiftSubDelay(0.1) }
    Shortcut { sequence: "A"; onActivated: { video.command(["cycle", "audio"]); root.osd("Piste audio suivante") } }
}
