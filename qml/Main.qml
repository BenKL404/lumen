import QtQuick
import QtQuick.Window
import QtQuick.Dialogs
import com.lumen.video
import com.lumen.player

Window {
    id: root

    width: 1280
    height: 760
    minimumWidth: 640
    minimumHeight: 400
    visible: true
    color: theme.background
    title: video.hasMedia ? video.mediaTitle + " — Lumen" : "Lumen"
    // Fenêtre sans bordure : la barre de titre est dessinée par Lumen
    flags: Qt.Window | Qt.FramelessWindowHint | (pinned ? Qt.WindowStaysOnTopHint : 0)

    // Design tokens : un seul endroit pour toute l'identité visuelle
    readonly property QtObject theme: QtObject {
        readonly property color background: "#0B0D10"
        readonly property color chrome: "#13161B"   // barres fixes (titre, contrôles, playlist)
        readonly property color menu: "#1A1E24"     // menus déroulants
        readonly property color surface: Qt.rgba(0.07, 0.08, 0.10, 0.86) // éléments flottants
        readonly property color surfaceHover: Qt.rgba(1, 1, 1, 0.08)
        readonly property color border: Qt.rgba(1, 1, 1, 0.08)
        readonly property color text: "#ECE8E1"
        readonly property color muted: "#8B9099"
        readonly property color track: Qt.rgba(1, 1, 1, 0.16)
        readonly property color accent: "#FF8C1A"
        readonly property int radius: 14
        readonly property int animation: 180
    }

    property bool controlsVisible: true
    property bool pinned: false
    property bool wasMaximized: false
    readonly property bool fullscreen: visibility === Window.FullScreen
    // En fenêtré, la playlist est ancrée à droite et réduit la zone vidéo ;
    // en plein écran, elle glisse par-dessus l'image.
    readonly property bool playlistDocked: playlistPanel.open && !fullscreen

    // Média en cours (clé de l'historique) ; vide tant que le fichier n'est pas chargé,
    // pour ne jamais enregistrer la position d'un fichier sous le nom d'un autre.
    property string currentUrl: ""
    property string pendingUrl: ""

    readonly property bool hasPrevious: playlist.previousIndex >= 0
    readonly property bool hasNext: playlist.nextIndex >= 0

    readonly property var menuEntries: [
        { label: "Ouvrir un fichier…", shortcut: "O", action: () => fileDialog.open() },
        { label: "Ajouter des sous-titres…", action: () => subtitleDialog.open() },
        { separator: true },
        { label: "Lecture / pause", shortcut: "Espace", action: () => video.togglePause() },
        { label: "Arrêter", action: () => root.stop() },
        { label: "Capture d'écran", shortcut: "S", action: () => root.screenshot() },
        { separator: true },
        { label: "Playlist", shortcut: "F6", checked: playlistPanel.open, action: () => root.togglePlaylist() },
        { label: "Plein écran", shortcut: "F", checked: root.fullscreen, action: () => root.toggleFullScreen() },
        { label: "Toujours au premier plan", checked: root.pinned, action: () => root.pinned = !root.pinned },
        { separator: true },
        { label: "Quitter", shortcut: "Ctrl+Q", action: () => root.close() }
    ]

    function wake() {
        controlsVisible = true
        hideTimer.restart()
    }

    function toggleFullScreen() {
        if (fullscreen) {
            visibility = wasMaximized ? Window.Maximized : Window.Windowed
        } else {
            wasMaximized = visibility === Window.Maximized
            visibility = Window.FullScreen
        }
    }

    function toggleMaximized() {
        visibility = visibility === Window.Maximized ? Window.Windowed : Window.Maximized
    }

    function togglePlaylist() {
        playlistPanel.open = !playlistPanel.open
    }

    function closePopups() {
        trackMenu.visible = false
        appMenu.visible = false
        if (fullscreen)
            playlistPanel.open = false
    }

    function osd(message) {
        osdText.text = message
        osdBox.opacity = 1
        osdTimer.restart()
    }

    function screenshot() {
        if (!video.hasMedia)
            return
        video.screenshot()
        osd("Capture enregistrée")
    }

    function openUrl(url) {
        saveProgress()
        currentUrl = ""
        pendingUrl = url.toString()
        playlist.load(pendingUrl)
        resumeBox.hide()
        video.loadFile(url)
        osd(utils.fileName(pendingUrl))
    }

    // Arrêt : ferme le fichier et revient à l'écran d'accueil (la playlist est conservée)
    function stop() {
        saveProgress()
        currentUrl = ""
        pendingUrl = ""
        resumeBox.hide()
        video.stop()
    }

    function playAt(index) {
        if (index >= 0 && index < playlist.items.length && index !== playlist.current)
            openUrl(playlist.items[index])
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
        appMenu.visible = false
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
    Playlist { id: playlist }

    // Sauvegarde régulière : la position survit à un plantage ou à une coupure
    Timer {
        id: saveTimer
        interval: 5000
        repeat: true
        running: video.hasMedia && !video.paused
        onTriggered: root.saveProgress()
    }

    Timer {
        id: hideTimer
        interval: 2500
        onTriggered: {
            if (video.hasMedia && !video.paused && !controls.hovered
                    && !trackMenu.visible && !appMenu.visible && !(root.fullscreen && playlistPanel.open))
                root.controlsVisible = false
        }
    }

    // ---------------------------------------------------- Barre de titre
    TitleBar {
        id: titleBar
        anchors { left: parent.left; right: parent.right; top: parent.top }
        visible: !root.fullscreen
        window: root
        theme: root.theme
        format: video.hasMedia ? utils.fileExtension(root.currentUrl) : ""
        title: video.hasMedia ? video.mediaTitle : "Lumen"
        pinned: root.pinned
        onMenuRequested: (anchor) => appMenu.popup(anchor, false)
        onPinToggled: root.pinned = !root.pinned
        onMaximizeToggled: root.toggleMaximized()
    }

    // ------------------------------------------------------ Zone vidéo
    Item {
        id: stage
        anchors {
            left: parent.left
            top: root.fullscreen ? parent.top : titleBar.bottom
            right: root.playlistDocked ? playlistPanel.left : parent.right
            bottom: root.fullscreen ? parent.bottom : controls.top
        }
        clip: true

        MpvVideo {
            id: video
            anchors.fill: parent
            onPausedChanged: {
                root.wake()
                if (paused)
                    root.saveProgress()
            }
            // Fin de la vidéo : enchaîner sur la suivante de la playlist
            onEofReachedChanged: if (eofReached && root.hasNext) root.playAt(playlist.nextIndex)
            onFileLoaded: {
                root.currentUrl = root.pendingUrl
                saveTimer.restart()
                root.resume()
            }
        }

        // Écran d'accueil
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

        // Titre en surimpression, en plein écran seulement (sinon il est dans la barre de titre)
        Rectangle {
            anchors { left: parent.left; right: parent.right; top: parent.top }
            height: 90
            visible: root.fullscreen && video.hasMedia
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

        // OSD
        Rectangle {
            id: osdBox
            anchors { top: parent.top; right: parent.right; margins: 20 }
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

        // Reprise de lecture
        Rectangle {
            id: resumeBox
            anchors { top: osdBox.bottom; right: parent.right; topMargin: 8; rightMargin: 20 }
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

        // Glisser-déposer
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
    }

    // ------------------------------------------------- Barre de contrôle
    ControlBar {
        id: controls
        floating: root.fullscreen
        anchors { left: parent.left; right: stage.right; bottom: parent.bottom; margins: floating ? 20 : 0 }
        video: video
        utils: utils
        theme: root.theme
        hasPrevious: root.hasPrevious
        hasNext: root.hasNext
        opacity: !floating || root.controlsVisible ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: theme.animation } }

        onOpenRequested: fileDialog.open()
        onStopRequested: root.stop()
        onFullscreenRequested: root.toggleFullScreen()
        onAudioMenuRequested: root.toggleTrackMenu("audio")
        onSubtitleMenuRequested: root.toggleTrackMenu("sub")
        onPreviousRequested: root.playAt(playlist.previousIndex)
        onNextRequested: root.playAt(playlist.nextIndex)
        onPlaylistRequested: root.togglePlaylist()
        onSettingsRequested: (anchor) => {
            trackMenu.visible = false
            appMenu.popup(anchor, true)
        }
    }

    // Un clic en dehors d'un menu (ou de la playlist flottante) le ferme, sans mettre en pause
    MouseArea {
        anchors.fill: parent
        enabled: trackMenu.visible || appMenu.visible || (root.fullscreen && playlistPanel.open)
        onClicked: root.closePopups()
    }

    // ------------------------------------------------------------- Playlist
    PlaylistPanel {
        id: playlistPanel
        anchors { top: root.fullscreen ? parent.top : titleBar.bottom; bottom: parent.bottom }
        width: root.fullscreen ? Math.min(400, root.width * 0.4) : 340
        x: open ? root.width - width : root.width
        visible: open || x < root.width
        Behavior on x {
            enabled: root.fullscreen
            NumberAnimation { duration: theme.animation; easing.type: Easing.OutCubic }
        }
        playlist: playlist
        utils: utils
        theme: root.theme
        onActivated: (index) => root.playAt(index)
        onAddRequested: addDialog.open()
    }

    // ------------------------------------------------------------ Menus
    TrackMenu {
        id: trackMenu
        anchors { right: controls.right; bottom: controls.top; bottomMargin: 10; rightMargin: controls.floating ? 0 : 10 }
        visible: false
        video: video
        utils: utils
        theme: root.theme
        onAddSubtitleRequested: {
            trackMenu.visible = false
            subtitleDialog.open()
        }
    }

    AppMenu {
        id: appMenu
        theme: root.theme
        entries: root.menuEntries
    }

    // ---------------------------------------- Redimensionnement (sans bordure)
    component ResizeEdge: MouseArea {
        property int edges
        enabled: Window.window && Window.window.visibility === Window.Windowed
        hoverEnabled: true
        onPressed: Window.window.startSystemResize(edges)
    }

    readonly property int grip: 6
    ResizeEdge { edges: Qt.LeftEdge; cursorShape: Qt.SizeHorCursor
        anchors { left: parent.left; top: parent.top; bottom: parent.bottom; topMargin: grip; bottomMargin: grip }
        width: grip }
    ResizeEdge { edges: Qt.RightEdge; cursorShape: Qt.SizeHorCursor
        anchors { right: parent.right; top: parent.top; bottom: parent.bottom; topMargin: grip; bottomMargin: grip }
        width: grip }
    ResizeEdge { edges: Qt.TopEdge; cursorShape: Qt.SizeVerCursor
        anchors { left: parent.left; right: parent.right; top: parent.top; leftMargin: grip; rightMargin: grip }
        height: grip }
    ResizeEdge { edges: Qt.BottomEdge; cursorShape: Qt.SizeVerCursor
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom; leftMargin: grip; rightMargin: grip }
        height: grip }
    ResizeEdge { edges: Qt.TopEdge | Qt.LeftEdge; cursorShape: Qt.SizeFDiagCursor
        anchors { left: parent.left; top: parent.top } width: grip * 2; height: grip * 2 }
    ResizeEdge { edges: Qt.TopEdge | Qt.RightEdge; cursorShape: Qt.SizeBDiagCursor
        anchors { right: parent.right; top: parent.top } width: grip * 2; height: grip * 2 }
    ResizeEdge { edges: Qt.BottomEdge | Qt.LeftEdge; cursorShape: Qt.SizeBDiagCursor
        anchors { left: parent.left; bottom: parent.bottom } width: grip * 2; height: grip * 2 }
    ResizeEdge { edges: Qt.BottomEdge | Qt.RightEdge; cursorShape: Qt.SizeFDiagCursor
        anchors { right: parent.right; bottom: parent.bottom } width: grip * 2; height: grip * 2 }

    // ------------------------------------------------------- Dialogues
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
        id: addDialog
        title: "Ajouter à la playlist"
        fileMode: FileDialog.OpenFiles
        currentFolder: root.currentUrl.substring(0, root.currentUrl.lastIndexOf("/"))
        nameFilters: fileDialog.nameFilters
        onAccepted: playlist.add(selectedFiles.map(url => url.toString()))
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
    Shortcut {
        sequence: "M"
        onActivated: { video.muted = !video.muted; root.osd(video.muted ? "Son rétabli" : "Muet") }
    }
    Shortcut { sequence: "F"; onActivated: root.toggleFullScreen() }
    Shortcut { sequence: "Return"; onActivated: root.toggleFullScreen() }
    Shortcut {
        sequence: "Escape"
        onActivated: {
            if (trackMenu.visible || appMenu.visible) {
                trackMenu.visible = false
                appMenu.visible = false
            } else if (playlistPanel.open) {
                playlistPanel.open = false
            } else if (root.fullscreen) {
                root.toggleFullScreen()
            }
        }
    }
    Shortcut { sequence: "O"; onActivated: fileDialog.open() }
    Shortcut { sequence: "S"; onActivated: root.screenshot() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: root.close() }
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
    Shortcut { sequence: "PgDown"; onActivated: root.playAt(playlist.nextIndex) }
    Shortcut { sequence: "PgUp"; onActivated: root.playAt(playlist.previousIndex) }
    Shortcut { sequence: "F6"; onActivated: root.togglePlaylist() }
    Shortcut {
        sequence: "Delete"
        enabled: playlistPanel.open
        onActivated: playlistPanel.removeSelected()
    }
    Shortcut { sequence: "Z"; onActivated: root.shiftSubDelay(-0.1) }
    Shortcut { sequence: "X"; onActivated: root.shiftSubDelay(0.1) }
    Shortcut { sequence: "A"; onActivated: { video.command(["cycle", "audio"]); root.osd("Piste audio suivante") } }
}
