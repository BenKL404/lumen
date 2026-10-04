import QtQuick
import QtQuick.Window
import QtQuick.Dialogs
import com.lumen.video
import com.lumen.player

Window {
    id: root

    // Taille de la dernière session (sans la playlist accolée, rouverte au besoin ensuite)
    width: settings.windowWidth
    height: settings.windowHeight
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
    // En fenêtré, la playlist est accolée au bord droit : la fenêtre s'élargit d'autant
    // et la vidéo garde sa taille. Fenêtre agrandie, elle prend place dans la fenêtre.
    property int attachedWidth: 0
    readonly property int playlistWidth: 360
    readonly property bool playlistAttached: playlistPanel.open && attachedWidth > 0 && visibility === Window.Windowed

    // Média en cours (clé de l'historique) ; vide tant que le fichier n'est pas chargé,
    // pour ne jamais enregistrer la position d'un fichier sous le nom d'un autre.
    property string currentUrl: ""
    property string pendingUrl: ""

    readonly property bool hasPrevious: playlist.previousIndex >= 0
    readonly property bool hasNext: playlist.nextIndex >= 0

    readonly property var menuEntries: [
        { label: "Ouvrir un fichier…", shortcut: "O", action: () => fileDialog.open() },
        { label: "Ouvrir un dossier…", shortcut: "Ctrl+O", action: () => folderDialog.open() },
        { label: "Ajouter des sous-titres…", action: () => subtitleDialog.open() },
        { separator: true },
        { label: "Lecture / pause", shortcut: "Espace", action: () => video.togglePause() },
        { label: "Arrêter", action: () => root.stop() },
        { label: "Capture d'écran", shortcut: "S", action: () => root.screenshot() },
        { separator: true },
        { label: "Playlist", shortcut: "F6", checked: playlistPanel.open, action: () => root.togglePlaylist() },
        { label: "Lecture aléatoire", shortcut: "H", checked: playlist.shuffle, action: () => root.toggleShuffle() },
        { label: "Répéter : " + root.repeatNames[playlist.repeatMode], shortcut: "R", checked: playlist.repeatMode !== 0, action: () => root.cycleRepeat() },
        { label: "Plein écran", shortcut: "F", checked: root.fullscreen, action: () => root.toggleFullScreen() },
        { label: "Toujours au premier plan", checked: root.pinned, action: () => root.pinned = !root.pinned },
        { separator: true },
        { label: "Reprendre là où je me suis arrêté", checked: settings.resumePlayback,
          action: () => root.toggleSetting("resumePlayback", "Reprise de la lecture") },
        { label: "Playlist automatique des épisodes", checked: settings.autoPlaylist,
          action: () => root.toggleSetting("autoPlaylist", "Playlist automatique") },
        { label: "Effacer l'historique de lecture", icon: "trash", action: () => { history.clear(); root.osd("Historique de lecture effacé") } },
        { separator: true },
        { label: "Quitter", shortcut: "Ctrl+Q", action: () => root.close() }
    ]

    function toggleSetting(name, label) {
        settings[name] = !settings[name]
        settings.save()
        osd(label + (settings[name] ? " activée" : " désactivée"))
    }

    // Paramètres de la session précédente, appliqués une fois la fenêtre créée
    Component.onCompleted: {
        video.volume = settings.volume
        video.muted = settings.muted
        imageKeys.forEach(k => image[k] = settings[k])
        playlist.setRepeat(settings.repeatMode)
        playlist.setShuffleEnabled(settings.shuffle)
        if (settings.maximized)
            visibility = Window.Maximized
        if (settings.playlistOpen)
            setPlaylistOpen(true)
    }

    function saveSettings() {
        settings.volume = video.volume
        settings.muted = video.muted
        settings.repeatMode = playlist.repeatMode
        settings.shuffle = playlist.shuffle
        settings.playlistOpen = playlistPanel.open
        imageKeys.forEach(k => settings[k] = image[k])
        settings.maximized = fullscreen ? wasMaximized : visibility === Window.Maximized
        // La taille n'est connue qu'en fenêtré ; sans la largeur de la playlist accolée
        if (visibility === Window.Windowed) {
            settings.windowWidth = width - attachedWidth
            settings.windowHeight = height
        }
        settings.save()
    }

    // Réglages d'image non observés côté mpv : suivis ici pour cocher le menu
    property string aspect: "-1"
    property int rotation: 0

    // Menu clic droit, construit à l'ouverture pour refléter l'état courant
    function contextEntries() {
        const has = video.hasMedia
        const audioTracks = video.tracks.filter(t => t.type === "audio")
        const subTracks = video.tracks.filter(t => t.type === "sub")
        const trackEntry = (t, property) => ({
            label: utils.trackLabel(t.id, t.title || "", t.lang || "", t.codec || "", t["demux-channel-count"] || 0),
            checked: t.selected === true,
            action: () => video.command(["set", property, String(t.id)])
        })
        const aspectEntry = (label, value) => ({
            label: label, checked: root.aspect === value,
            action: () => { root.aspect = value; video.command(["set", "video-aspect-override", value]); root.osd("Format d'image : " + label) }
        })

        return [
            { label: "Ouvrir un fichier…", icon: "folder-open", shortcut: "O", action: () => fileDialog.open() },
            { label: "Ouvrir un dossier…", icon: "folder", shortcut: "Ctrl+O", action: () => folderDialog.open() },
            { label: "Ajouter à la playlist…", action: () => addDialog.open() },
            { separator: true },
            { label: "Lecture", icon: "play", enabled: has, submenu: [
                { label: video.paused ? "Lecture" : "Pause", icon: video.paused ? "play" : "pause", shortcut: "Espace", action: () => video.togglePause() },
                { label: "Arrêter", icon: "stop", action: () => root.stop() },
                { separator: true },
                { label: "Précédent", icon: "skip-back", shortcut: "PgUp", enabled: root.hasPrevious, action: () => root.playAt(playlist.previousIndex) },
                { label: "Suivant", icon: "skip-forward", shortcut: "PgDn", enabled: root.hasNext, action: () => root.playAt(playlist.nextIndex) },
                { separator: true },
                { label: root.abLoopLabel(), icon: "repeat", shortcut: "L", checked: video.abLoopB >= 0, action: () => root.cycleAbLoop() },
                { separator: true },
                { label: "Lecture aléatoire", icon: "shuffle", shortcut: "H", checked: playlist.shuffle, action: () => root.toggleShuffle() },
                { label: "Ne pas répéter", checked: playlist.repeatMode === 0, action: () => root.setRepeat(0) },
                { label: "Répéter le fichier", checked: playlist.repeatMode === 1, action: () => root.setRepeat(1) },
                { label: "Répéter la playlist", checked: playlist.repeatMode === 2, action: () => root.setRepeat(2) },
                { separator: true },
                { label: "Reculer de 5 s", shortcut: "←", action: () => root.seekBy(-5) },
                { label: "Avancer de 5 s", shortcut: "→", action: () => root.seekBy(5) },
                { label: "Reculer de 30 s", shortcut: "Ctrl+←", action: () => root.seekBy(-30) },
                { label: "Avancer de 30 s", shortcut: "Ctrl+→", action: () => root.seekBy(30) },
                { label: "Image précédente", shortcut: ",", action: () => video.frameStep(false) },
                { label: "Image suivante", shortcut: ".", action: () => video.frameStep(true) }
            ] },
            { label: "Chapitres", icon: "list", enabled: has && video.chapters.length > 0, submenu:
                video.chapters.map((c, i) => ({
                    label: (i + 1) + ". " + (c.title || "Chapitre " + (i + 1)),
                    // Arrondi : les débuts de chapitre MKV tombent souvent juste avant la seconde
                    shortcut: utils.formatTime(Math.round(c.time)),
                    checked: i === video.chapter,
                    action: () => root.goToChapter(i)
                })).concat([
                    { separator: true },
                    { label: "Chapitre précédent", shortcut: "Ctrl+PgUp", action: () => root.stepChapter(-1) },
                    { label: "Chapitre suivant", shortcut: "Ctrl+PgDn", action: () => root.stepChapter(1) }
                ])
            },
            { label: "Signets", icon: "pin", enabled: has, submenu:
                [{ label: "Ajouter un signet ici", icon: "plus", shortcut: "B", action: () => root.addBookmark() }]
                .concat(root.bookmarks.length > 0 ? [{ separator: true }] : [],
                        root.bookmarks.map((t, i) => ({
                            label: "Signet " + (i + 1), shortcut: utils.formatTime(t),
                            action: () => video.seekAbsolute(t)
                        })),
                        root.bookmarks.length > 0
                            ? [{ separator: true }, { label: "Supprimer tous les signets", icon: "trash", action: () => root.clearBookmarks() }]
                            : [])
            },
            { label: "Vitesse", icon: "gauge", enabled: has, submenu:
                [0.5, 0.75, 1, 1.25, 1.5, 2].map(v => ({
                    label: v === 1 ? "×1 (normale)" : "×" + String(v).replace(".", ","),
                    shortcut: v === 1 ? "Retour arr." : "",
                    checked: Math.abs(video.speed - v) < 0.01,
                    action: () => root.setSpeed(v)
                }))
            },
            { label: "Audio", icon: "audio-lines", enabled: has, submenu:
                (audioTracks.length > 0 ? audioTracks.map(t => trackEntry(t, "aid"))
                                        : [{ label: "Aucune piste audio", enabled: false }]).concat([
                { separator: true },
                { label: "Muet", shortcut: "M", checked: video.muted, action: () => root.toggleMute() },
                { label: "Augmenter le volume", shortcut: "↑", action: () => root.changeVolume(5) },
                { label: "Baisser le volume", shortcut: "↓", action: () => root.changeVolume(-5) }
            ]) },
            { label: "Sous-titres", icon: "captions", enabled: has, submenu:
                [{ label: "Désactivés", checked: !subTracks.some(t => t.selected), action: () => video.command(["set", "sid", "no"]) }]
                .concat(subTracks.map(t => trackEntry(t, "sid")), [
                { separator: true },
                { label: "Ajouter un fichier…", action: () => subtitleDialog.open() },
                { label: "Décaler de −0,1 s", shortcut: "Z", action: () => root.shiftSubDelay(-0.1) },
                { label: "Décaler de +0,1 s", shortcut: "X", action: () => root.shiftSubDelay(0.1) },
                { label: "Réinitialiser le décalage", enabled: video.subDelay !== 0, action: () => root.shiftSubDelay(-video.subDelay) }
            ]) },
            { label: "Vidéo", icon: "monitor", enabled: has, submenu: [
                { label: "Plein écran", icon: "maximize", shortcut: "F", checked: root.fullscreen, action: () => root.toggleFullScreen() },
                { label: "Capture d'écran", icon: "camera", shortcut: "S", action: () => root.screenshot() },
                { label: "Réglages d'image…", icon: "settings", shortcut: "I", action: () => root.toggleImagePanel() },
                { separator: true },
                aspectEntry("Format automatique", "-1"),
                aspectEntry("16:9", "16:9"),
                aspectEntry("4:3", "4:3"),
                aspectEntry("2,35:1", "2.35:1"),
                { separator: true },
                { label: "Pivoter de 90°", icon: "rotate-cw", checked: root.rotation !== 0, action: () => root.rotate() }
            ] },
            { separator: true },
            { label: "Playlist", icon: "list", shortcut: "F6", checked: playlistPanel.open, action: () => root.togglePlaylist() },
            { label: "Toujours au premier plan", icon: "pin", checked: root.pinned, action: () => root.pinned = !root.pinned },
            { separator: true },
            { label: "Quitter", shortcut: "Ctrl+Q", action: () => root.close() }
        ]
    }

    // --------------------------------------------------- Réglages d'image
    // Appliqués à mpv dès qu'ils changent ; réglages globaux, conservés d'un fichier
    // à l'autre et d'une session à l'autre (settings.toml)
    readonly property QtObject image: QtObject {
        property int brightness: 0
        property int contrast: 0
        property int saturation: 0
        property int gamma: 0
        property int hue: 0
        property int zoom: 100

        onBrightnessChanged: video.command(["set", "brightness", String(brightness)])
        onContrastChanged: video.command(["set", "contrast", String(contrast)])
        onSaturationChanged: video.command(["set", "saturation", String(saturation)])
        onGammaChanged: video.command(["set", "gamma", String(gamma)])
        onHueChanged: video.command(["set", "hue", String(hue)])
        // mpv attend un zoom logarithmique : 0 = taille normale, 1 = ×2
        onZoomChanged: video.command(["set", "video-zoom", String(Math.log2(zoom / 100))])
    }

    readonly property var imageKeys: ["brightness", "contrast", "saturation", "gamma", "hue", "zoom"]

    function toggleImagePanel() {
        trackMenu.visible = false
        appMenu.close()
        contextMenu.close()
        imagePanel.visible = !imagePanel.visible
    }

    // ------------------------------------------- Chapitres, signets, boucle A-B
    property var bookmarks: []

    function refreshBookmarks() {
        bookmarks = currentUrl !== "" ? history.bookmarks(currentUrl) : []
    }

    function addBookmark() {
        if (currentUrl === "")
            return
        history.addBookmark(currentUrl, video.position)
        refreshBookmarks()
        osd("Signet ajouté à " + utils.formatTime(video.position))
    }

    function clearBookmarks() {
        history.clearBookmarks(currentUrl)
        refreshBookmarks()
        osd("Signets supprimés")
    }

    function goToChapter(index) {
        const chapters = video.chapters
        if (index < 0 || index >= chapters.length)
            return
        video.command(["set", "chapter", String(index)])
        osd("Chapitre " + (index + 1) + (chapters[index].title ? " : " + chapters[index].title : ""))
    }

    function stepChapter(step) {
        if (video.chapters.length === 0)
            return
        // Reculer pendant les premières secondes d'un chapitre ramène au précédent,
        // sinon au début du chapitre en cours (comme les pistes d'un CD)
        let index = video.chapter + step
        if (step < 0 && video.chapter >= 0
                && video.position - video.chapters[video.chapter].time > 3)
            index = video.chapter
        goToChapter(Math.max(0, Math.min(video.chapters.length - 1, index)))
    }

    // Premier appui : début A ; deuxième : fin B (la boucle démarre) ; troisième : désactivée
    function cycleAbLoop() {
        if (!video.hasMedia)
            return
        if (video.abLoopA < 0) {
            video.command(["set", "ab-loop-a", String(video.position)])
            osd("Boucle : début A à " + utils.formatTime(video.position))
        } else if (video.abLoopB < 0) {
            let a = video.abLoopA, b = video.position
            if (b < a)
                [a, b] = [b, a]
            video.command(["set", "ab-loop-a", String(a)])
            video.command(["set", "ab-loop-b", String(b)])
            osd("Boucle A-B : " + utils.formatTime(a) + " → " + utils.formatTime(b))
        } else {
            clearAbLoop()
        }
    }

    function clearAbLoop() {
        video.command(["set", "ab-loop-a", "no"])
        video.command(["set", "ab-loop-b", "no"])
        osd("Boucle A-B désactivée")
    }

    function abLoopLabel() {
        return video.abLoopA < 0 ? "Boucle A-B : définir le début (A)"
             : video.abLoopB < 0 ? "Boucle A-B : définir la fin (B)"
             : "Désactiver la boucle A-B"
    }

    function openContextMenu(item, x, y) {
        trackMenu.visible = false
        appMenu.close()
        contextMenu.entries = contextEntries()
        const p = item.mapToItem(contextMenu.parent, x, y)
        contextMenu.popupAt(p.x, p.y)
    }

    readonly property var repeatNames: ["désactivé", "le fichier", "la playlist"]

    function cycleRepeat() {
        playlist.cycleRepeat()
        osd(playlist.repeatMode === 0 ? "Répétition désactivée" : "Répéter " + repeatNames[playlist.repeatMode])
    }

    function setRepeat(mode) {
        while (playlist.repeatMode !== mode)
            playlist.cycleRepeat()
        osd(mode === 0 ? "Répétition désactivée" : "Répéter " + repeatNames[mode])
    }

    function toggleShuffle() {
        playlist.toggleShuffle()
        osd(playlist.shuffle ? "Lecture aléatoire" : "Lecture dans l'ordre")
    }

    function openFolder(url) {
        const first = playlist.openFolder(url.toString())
        if (first === "") {
            osd("Aucun fichier audio ou vidéo dans ce dossier")
            return
        }
        openUrl(first)
        setPlaylistOpen(true)
    }

    function seekBy(seconds) {
        video.seekRelative(seconds)
        osd((seconds > 0 ? "+" : "−") + Math.abs(seconds) + " s")
    }

    function changeVolume(step) {
        const volume = Math.max(0, Math.min(130, video.volume + step))
        video.volume = volume
        osd("Volume " + Math.round(volume) + " %")
    }

    function toggleMute() {
        osd(video.muted ? "Son rétabli" : "Muet")
        video.muted = !video.muted
    }

    function setSpeed(speed) {
        video.speed = speed
        osd(Math.abs(speed - 1) < 0.01 ? "Vitesse normale" : "Vitesse ×" + speed.toFixed(2).replace(/0$/, "").replace(".", ","))
    }

    function rotate() {
        rotation = (rotation + 90) % 360
        video.command(["set", "video-rotate", String(rotation)])
        osd("Rotation " + rotation + "°")
    }

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
        setPlaylistOpen(!playlistPanel.open)
    }

    function setPlaylistOpen(open) {
        if (open === playlistPanel.open)
            return
        if (visibility === Window.Windowed) {
            if (open) {
                attachedWidth = playlistWidth
                width += attachedWidth
            } else if (attachedWidth > 0) {
                width -= attachedWidth
            }
        }
        if (!open)
            attachedWidth = 0
        playlistPanel.open = open
    }

    function closePopups() {
        trackMenu.visible = false
        imagePanel.visible = false
        appMenu.close()
        contextMenu.close()
        if (fullscreen)
            setPlaylistOpen(false)
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
        if (utils.isFolder(url.toString())) {
            openFolder(url)
            return
        }
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
        bookmarks = []
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
        if (currentUrl === "" || !settings.resumePlayback)
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
        imagePanel.visible = false
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
        if (!found || !settings.resumePlayback)
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

    onClosing: {
        saveProgress()
        saveSettings()
    }

    Utils { id: utils }
    History { id: history }
    Settings { id: settings }

    // Pas de mise en veille de l'écran pendant la lecture d'une vidéo ; en pause, à
    // l'arrêt ou pour un fichier audio, le bureau reprend la main
    ScreenGuard { id: screenGuard }
    readonly property bool keepScreenAwake: video.hasMedia && !video.paused && !video.eofReached && controls.realVideo
    onKeepScreenAwakeChanged: screenGuard.setActive(keepScreenAwake)

    // Contrôle par le bureau (MPRIS) : touches multimédia, applet du panneau, écran de verrouillage
    Mpris {
        id: mpris
        onCommand: (name, number, text) => root.handleDesktopCommand(name, number, text)
    }

    function publishState() {
        const status = !video.hasMedia ? 0 : video.paused ? 2 : 1
        mpris.update(status, video.mediaTitle || utils.fileName(currentUrl), currentUrl,
                     video.duration, video.position, video.volume, video.speed,
                     playlist.repeatMode, playlist.shuffle, hasNext, hasPrevious)
    }

    function handleDesktopCommand(name, number, text) {
        switch (name) {
        case "play-pause": if (video.hasMedia) video.togglePause(); break
        case "play": video.paused = false; break
        case "pause": video.paused = true; break
        case "stop": stop(); break
        case "next": playAt(playlist.nextIndex); break
        case "previous": playAt(playlist.previousIndex); break
        case "seek": video.seekRelative(number); break
        case "position": video.seekAbsolute(number); break
        case "volume": video.volume = number; break
        case "rate": setSpeed(Math.max(0.25, Math.min(4, number))); break
        case "repeat": playlist.setRepeat(number); break
        case "shuffle": playlist.setShuffleEnabled(number > 0); break
        case "raise": raise(); requestActivate(); break
        case "quit": close(); break
        case "open": openUrl(text); break
        }
        Qt.callLater(publishState)
    }

    // Position une fois par seconde (le bureau l'interroge) ; le reste dès qu'il change
    Timer {
        interval: 1000
        repeat: true
        running: true
        triggeredOnStart: true
        onTriggered: root.publishState()
    }
    Connections {
        target: video
        function onPausedChanged() { Qt.callLater(root.publishState) }
        function onHasMediaChanged() { Qt.callLater(root.publishState) }
        function onMediaTitleChanged() { Qt.callLater(root.publishState) }
        function onDurationChanged() { Qt.callLater(root.publishState) }
        function onVolumeChanged() { Qt.callLater(root.publishState) }
        function onSpeedChanged() { Qt.callLater(root.publishState) }
    }
    Connections {
        target: playlist
        function onRepeatModeChanged() { Qt.callLater(root.publishState) }
        function onShuffleChanged() { Qt.callLater(root.publishState) }
        function onNextIndexChanged() { Qt.callLater(root.publishState) }
        function onPreviousIndexChanged() { Qt.callLater(root.publishState) }
    }
    Playlist {
        id: playlist
        autoBuild: settings.autoPlaylist
        // « Répéter le fichier » est confié à mpv : la fin du fichier n'est alors jamais atteinte
        onRepeatModeChanged: video.command(["set", "loop-file", repeatMode === 1 ? "inf" : "no"])
    }

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
                    && !trackMenu.visible && !imagePanel.visible && !appMenu.visible && !contextMenu.visible && !(root.fullscreen && playlistPanel.open))
                root.controlsVisible = false
        }
    }

    // ---------------------------------------------------- Barre de titre
    TitleBar {
        id: titleBar
        anchors { left: parent.left; right: root.playlistAttached ? playlistPanel.left : parent.right; top: parent.top }
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
            onEofReachedChanged: {
                if (!eofReached || playlist.nextIndex < 0)
                    return
                // Répéter une playlist d'un seul fichier : le relancer depuis le début
                if (playlist.nextIndex === playlist.current) {
                    video.seekAbsolute(0)
                    video.paused = false
                } else {
                    root.playAt(playlist.nextIndex)
                }
            }
            onFileLoaded: {
                root.currentUrl = root.pendingUrl
                saveTimer.restart()
                root.refreshBookmarks()
                // La boucle A-B est un réglage global de mpv : ne pas la garder d'un fichier à l'autre
                if (video.abLoopA >= 0 || video.abLoopB >= 0) {
                    video.command(["set", "ab-loop-a", "no"])
                    video.command(["set", "ab-loop-b", "no"])
                }
                root.resume()
            }
        }

        MouseArea {
            anchors.fill: parent
            id: stageArea
            hoverEnabled: true
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            cursorShape: root.controlsVisible ? Qt.ArrowCursor : Qt.BlankCursor

            onPositionChanged: root.wake()
            // Clic droit : menu. Clic gauche : compté (voir clickTimer)
            onPressed: (mouse) => {
                if (mouse.button === Qt.LeftButton) {
                    clickTimer.clicks += 1
                    clickTimer.restart()
                }
            }
            onClicked: (mouse) => {
                if (mouse.button === Qt.RightButton)
                    root.openContextMenu(stageArea, mouse.x, mouse.y)
            }
            onWheel: (wheel) => root.changeVolume(wheel.angleDelta.y > 0 ? 5 : -5)

            // Double-clic : lecture / pause ; triple-clic : plein écran ; simple clic : rien
            // (pas de pause par accident). On attend la fin de la série de clics pour savoir
            // si un double-clic va devenir un triple-clic.
            Timer {
                id: clickTimer

                property int clicks: 0

                interval: 250
                onTriggered: {
                    if (clicks >= 3)
                        root.toggleFullScreen()
                    else if (clicks === 2 && video.hasMedia)
                        video.togglePause()
                    clicks = 0
                }
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

        // OSD : texte seul sur l'image, sans fond ; un fin contour sombre le garde
        // lisible sur les scènes claires
        Item {
            id: osdBox
            anchors { top: parent.top; right: parent.right; margins: 20 }
            width: osdText.implicitWidth
            height: osdText.implicitHeight
            opacity: 0
            Behavior on opacity { NumberAnimation { duration: theme.animation } }

            Text {
                id: osdText
                color: theme.text
                font.pixelSize: 20
                font.weight: Font.DemiBold
                style: Text.Outline
                styleColor: Qt.rgba(0, 0, 0, 0.75)
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
        repeatMode: playlist.repeatMode
        shuffle: playlist.shuffle
        onRepeatRequested: root.cycleRepeat()
        onShuffleRequested: root.toggleShuffle()
        bookmarks: root.bookmarks
        onAbLoopClearRequested: root.clearAbLoop()
        imagePanelOpen: imagePanel.visible
        mediaUrl: root.currentUrl
        onImageSettingsRequested: root.toggleImagePanel()
    }

    // Un clic en dehors d'un menu (ou de la playlist flottante) le ferme, sans mettre en pause
    MouseArea {
        anchors.fill: parent
        enabled: trackMenu.visible || imagePanel.visible || appMenu.visible || contextMenu.visible
                 || (root.fullscreen && playlistPanel.open)
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.closePopups()
    }

    // ------------------------------------------------------------- Playlist
    PlaylistPanel {
        id: playlistPanel
        anchors { top: root.fullscreen || root.playlistAttached ? parent.top : titleBar.bottom; bottom: parent.bottom }
        width: root.fullscreen ? Math.min(400, root.width * 0.4) : root.playlistWidth
        x: open ? root.width - width : root.width
        visible: open || x < root.width
        Behavior on x {
            enabled: root.fullscreen
            NumberAnimation { duration: theme.animation; easing.type: Easing.OutCubic }
        }
        playlist: playlist
        utils: utils
        theme: root.theme
        window: root
        attached: root.playlistAttached
        onActivated: (index) => root.playAt(index)
        // « Ajouter » : des fichiers ou tout un dossier
        onAddRequested: (anchor) => {
            trackMenu.visible = false
            appMenu.close()
            contextMenu.entries = [
                { label: "Des fichiers…", icon: "folder-open", action: () => addDialog.open() },
                { label: "Un dossier…", icon: "folder", action: () => addFolderDialog.open() }
            ]
            contextMenu.popup(anchor, true)
        }
        onCloseRequested: root.setPlaylistOpen(false)
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

    ImagePanel {
        id: imagePanel
        anchors { right: controls.right; bottom: controls.top; bottomMargin: 10; rightMargin: controls.floating ? 0 : 10 }
        visible: false
        theme: root.theme
        image: root.image
        onCloseRequested: visible = false
    }

    AppMenu {
        id: appMenu
        theme: root.theme
        entries: root.menuEntries
    }

    AppMenu {
        id: contextMenu
        theme: root.theme
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

    FolderDialog {
        id: folderDialog
        title: "Ouvrir un dossier"
        onAccepted: root.openFolder(selectedFolder)
    }

    FolderDialog {
        id: addFolderDialog
        title: "Ajouter un dossier à la playlist"
        onAccepted: playlist.addFolder(selectedFolder.toString())
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
    Shortcut { sequence: "Left"; onActivated: root.seekBy(-5) }
    Shortcut { sequence: "Right"; onActivated: root.seekBy(5) }
    Shortcut { sequence: "Ctrl+Left"; onActivated: root.seekBy(-30) }
    Shortcut { sequence: "Ctrl+Right"; onActivated: root.seekBy(30) }
    Shortcut { sequence: "Up"; onActivated: root.changeVolume(5) }
    Shortcut { sequence: "Down"; onActivated: root.changeVolume(-5) }
    Shortcut { sequence: "M"; onActivated: root.toggleMute() }
    Shortcut { sequence: "F"; onActivated: root.toggleFullScreen() }
    Shortcut { sequence: "Return"; onActivated: root.toggleFullScreen() }
    Shortcut {
        sequence: "Escape"
        onActivated: {
            if (trackMenu.visible || imagePanel.visible || appMenu.visible || contextMenu.visible) {
                root.closePopups()
            } else if (playlistPanel.open) {
                root.setPlaylistOpen(false)
            } else if (root.fullscreen) {
                root.toggleFullScreen()
            }
        }
    }
    Shortcut { sequence: "O"; onActivated: fileDialog.open() }
    Shortcut { sequence: "Ctrl+O"; onActivated: folderDialog.open() }
    Shortcut { sequence: "R"; onActivated: root.cycleRepeat() }
    Shortcut { sequence: "L"; onActivated: root.cycleAbLoop() }
    Shortcut { sequence: "I"; onActivated: root.toggleImagePanel() }
    Shortcut { sequence: "B"; onActivated: root.addBookmark() }
    Shortcut { sequence: "Ctrl+PgUp"; onActivated: root.stepChapter(-1) }
    Shortcut { sequence: "Ctrl+PgDown"; onActivated: root.stepChapter(1) }
    Shortcut { sequence: "H"; onActivated: root.toggleShuffle() }
    Shortcut { sequence: "S"; onActivated: root.screenshot() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: root.close() }
    Shortcut { sequence: "."; onActivated: video.frameStep(true) }
    Shortcut { sequence: ","; onActivated: video.frameStep(false) }
    Shortcut { sequence: "]"; onActivated: root.setSpeed(Math.min(4, Math.round((video.speed + 0.1) * 10) / 10)) }
    Shortcut { sequence: "["; onActivated: root.setSpeed(Math.max(0.25, Math.round((video.speed - 0.1) * 10) / 10)) }
    Shortcut { sequence: "Backspace"; onActivated: root.setSpeed(1) }
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
