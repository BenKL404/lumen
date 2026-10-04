import QtQuick

// Icône vectorielle (tracés du jeu Lucide, grille 24×24, trait de 2 px, licence ISC).
// Le SVG est généré à la volée dans la couleur demandée : pas de fichier ni de module en plus.
Item {
    id: icon

    property string name
    property color color: "white"
    property real size: 18
    property real strokeWidth: 2
    // Formes pleines (lecture, pause…) plutôt qu'en contour
    property bool filled: false

    // Contenu SVG de chaque icône, dans un repère 24×24
    readonly property var shapes: ({
        "play": '<path d="M6 4l14 8-14 8z"/>',
        "pause": '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>',
        "stop": '<rect x="5" y="5" width="14" height="14" rx="2"/>',
        "skip-back": '<path d="M19 20L9 12l10-8z"/><path d="M5 19V5"/>',
        "skip-forward": '<path d="M5 4l10 8-10 8z"/><path d="M19 5v14"/>',
        "folder-open": '<path d="M6 14l1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"/>',
        "folder": '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2z"/>',
        "volume": '<path d="M11 5L6 9H2v6h4l5 4z"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>',
        "volume-x": '<path d="M11 5L6 9H2v6h4l5 4z"/><path d="M22 9l-6 6"/><path d="M16 9l6 6"/>',
        "audio-lines": '<path d="M2 10v3M6 6v11M10 3v18M14 8v7M18 5v13M22 10v3"/>',
        "music": '<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>',
        "captions": '<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 15h4M15 15h2M7 11h2M13 11h4"/>',
        "gauge": '<path d="M12 14l4-4"/><path d="M3.34 19a10 10 0 1 1 17.32 0"/>',
        "camera": '<path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3z"/><circle cx="12" cy="13" r="3"/>',
        "settings": '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
        "sliders": '<path d="M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4"/>',
        "shuffle": '<path d="M2 18h1.4c1.3 0 2.5-.6 3.3-1.7l6.1-8.6c.7-1.1 2-1.7 3.3-1.7H22"/><path d="M18 2l4 4-4 4"/><path d="M2 6h1.9c1.5 0 2.9.9 3.6 2.2"/><path d="M22 18h-5.9c-1.3 0-2.6-.7-3.3-1.8l-.5-.8"/><path d="M18 14l4 4-4 4"/>',
        "repeat": '<path d="M17 2l4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="M7 22l-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/>',
        "repeat-1": '<path d="M17 2l4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="M7 22l-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/><path d="M11 10h1v4"/>',
        "list": '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
        "maximize": '<path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3"/>',
        "minimize": '<path d="M8 3v3a2 2 0 0 1-2 2H3M21 8h-3a2 2 0 0 1-2-2V3M3 16h3a2 2 0 0 1 2 2v3M16 21v-3a2 2 0 0 1 2-2h3"/>',
        "pin": '<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>',
        "window-minimize": '<path d="M5 12h14"/>',
        "window-maximize": '<rect x="4" y="4" width="16" height="16" rx="1"/>',
        "window-restore": '<rect x="4" y="8" width="12" height="12" rx="1"/><path d="M8 8V5a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1h-3"/>',
        "x": '<path d="M18 6L6 18M6 6l12 12"/>',
        "check": '<path d="M20 6L9 17l-5-5"/>',
        "chevron-down": '<path d="M6 9l6 6 6-6"/>',
        "chevron-up": '<path d="M18 15l-6-6-6 6"/>',
        "chevron-right": '<path d="M9 18l6-6-6-6"/>',
        "arrow-up-to-line": '<path d="M5 3h14M18 13l-6-6-6 6M12 7v14"/>',
        "arrow-down-to-line": '<path d="M12 17V3M6 11l6 6 6-6M19 21H5"/>',
        "search": '<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
        "plus": '<path d="M5 12h14M12 5v14"/>',
        "minus": '<path d="M5 12h14"/>',
        "rotate-cw": '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>',
        "rotate-ccw": '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>',
        "monitor": '<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>',
        "trash": '<path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>'
    })

    readonly property string svg: {
        const body = shapes[name]
        if (body === undefined)
            return ""
        // SVG ne connaît pas #aarrggbb : couleur en rgb(), transparence via l'opacité de l'image
        const c = "rgb(" + Math.round(color.r * 255) + "," + Math.round(color.g * 255) + "," + Math.round(color.b * 255) + ")"
        return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="' + (filled ? c : "none")
             + '" stroke="' + c + '" stroke-width="' + strokeWidth
             + '" stroke-linecap="round" stroke-linejoin="round">' + body + '</svg>'
    }

    implicitWidth: size
    implicitHeight: size

    Image {
        anchors.centerIn: parent
        width: icon.size
        height: icon.size
        sourceSize: Qt.size(icon.size, icon.size)
        source: icon.svg === "" ? "" : "data:image/svg+xml;base64," + Qt.btoa(icon.svg)
        opacity: icon.color.a
        smooth: true
        cache: true
    }
}
