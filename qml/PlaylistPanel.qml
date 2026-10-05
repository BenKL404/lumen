import QtQuick

// Playlist. En mode fenêtré, elle est accolée au bord droit de la fenêtre comme une
// extension (la fenêtre s'élargit, la vidéo garde sa taille) ; fenêtre agrandie, elle
// réduit la zone vidéo ; en plein écran, elle flotte par-dessus l'image.
// Clic : sélectionner ; double-clic : lire ; barre du bas : réorganiser, ajouter, retirer, trier, rechercher.
Rectangle {
    id: panel

    property var playlist
    property var utils
    property var theme
    property var window
    property bool open: false
    // Accolée à la fenêtre : a sa propre barre de titre, qui sert aussi à déplacer la fenêtre
    property bool attached: false
    property int selected: -1
    // Glisser-déposer : ligne d'insertion (index d'arrivée), -1 hors glissement
    property int dropIndex: -1
    property string filter: ""

    readonly property int count: playlist.items.length
    readonly property real totalDuration: playlist.durations.reduce((sum, d) => d > 0 ? sum + d : sum, 0)

    signal activated(int index)
    signal addRequested(Item anchor)
    signal closeRequested()

    function matches(index) {
        return filter === "" || utils.fileName(playlist.items[index]).toLowerCase().indexOf(filter.toLowerCase()) >= 0
    }

    function moveSelected(to) {
        if (selected < 0 || selected >= count)
            return
        to = Math.max(0, Math.min(count - 1, to))
        playlist.moveItem(selected, to)
        selected = to
        list.positionViewAtIndex(selected, ListView.Contain)
    }

    function removeSelected() {
        if (selected < 0 || selected >= count)
            return
        playlist.remove(selected)
        selected = Math.min(selected, count - 1)
    }

    function closeSearch() {
        search.text = ""
        search.focus = false
        toolbar.searching = false
    }

    color: theme.chrome

    // Absorbe les clics : ne pas mettre en pause en cliquant dans le panneau
    MouseArea { anchors.fill: parent }

    onOpenChanged: {
        if (open) {
            selected = playlist.current
            list.positionViewAtIndex(Math.max(0, playlist.current), ListView.Center)
        } else {
            closeSearch()
        }
    }

    // Séparation avec la fenêtre principale
    Rectangle {
        anchors { left: parent.left; top: parent.top; bottom: parent.bottom }
        width: panel.attached ? 2 : 1
        color: panel.attached ? panel.theme.divider : panel.theme.border
        z: 1
    }

    // ------------------------------------------------------------ En-tête
    Rectangle {
        id: header
        anchors { left: parent.left; right: parent.right; top: parent.top }
        height: 36
        color: panel.attached ? panel.theme.chrome : "transparent"

        DragHandler {
            target: null
            enabled: panel.attached
            onActiveChanged: if (active) panel.window.startSystemMove()
        }

        Text {
            anchors { left: parent.left; leftMargin: 14; verticalCenter: parent.verticalCenter }
            text: "Playlist"
            color: panel.theme.text
            font.pixelSize: 13
            font.weight: Font.DemiBold
        }
        Text {
            anchors { right: closeButton.left; rightMargin: 6; verticalCenter: parent.verticalCenter }
            text: (panel.playlist.current >= 0 ? (panel.playlist.current + 1) + " / " : "") + panel.count
                  + (panel.totalDuration > 0 ? "  ·  " + panel.utils.formatTime(panel.totalDuration) : "")
            color: panel.theme.muted
            font.pixelSize: 12
            font.family: "monospace"
        }
        Rectangle {
            id: closeButton
            anchors { right: parent.right; top: parent.top; bottom: parent.bottom }
            width: 42
            color: closeArea.containsMouse ? "#C42B1C" : "transparent"

            Icon {
                anchors.centerIn: parent
                name: "x"
                size: 15
                strokeWidth: 1.8
                color: panel.theme.text
            }
            MouseArea {
                id: closeArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: panel.closeRequested()
            }
        }
        Rectangle {
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
            height: 1
            color: panel.theme.border
        }
    }

    // ------------------------------------------------------------ Liste
    ListView {
        id: list
        anchors { left: parent.left; right: parent.right; top: header.bottom; bottom: toolbar.top; margins: 6 }
        clip: true
        model: panel.playlist.items
        boundsBehavior: Flickable.StopAtBounds
        // Pas de défilement par glissement : le glisser sert à réorganiser
        interactive: panel.dropIndex < 0

        // Ligne d'insertion pendant le glisser-déposer
        Rectangle {
            parent: list.contentItem
            visible: panel.dropIndex >= 0
            x: 6
            y: panel.dropIndex * 30 + (panel.dropIndex > panel.selected ? 29 : -1)
            width: list.width - 12
            height: 2
            radius: 1
            color: panel.theme.accent
            z: 10
        }

        delegate: Rectangle {
            id: row

            required property string modelData
            required property int index
            readonly property bool isCurrent: index === panel.playlist.current
            readonly property bool isSelected: index === panel.selected
            readonly property real duration: panel.playlist.durations[index] ?? -1

            width: ListView.view.width
            visible: panel.matches(index)
            height: visible ? 30 : 0
            radius: 6
            color: isSelected ? panel.theme.raised
                 : rowArea.containsMouse ? panel.theme.subtle : "transparent"

            // Repère du fichier en cours
            Rectangle {
                anchors { left: parent.left; top: parent.top; bottom: parent.bottom; margins: 7 }
                width: 3
                radius: 1.5
                color: panel.theme.accent
                visible: row.isCurrent
            }
            Text {
                id: number
                anchors { left: parent.left; leftMargin: 16; verticalCenter: parent.verticalCenter }
                width: 28
                text: String(row.index + 1).padStart(2, "0") + "."
                color: row.isCurrent ? panel.theme.accent : panel.theme.muted
                font.pixelSize: 12
                font.family: "monospace"
            }
            Text {
                anchors { left: number.right; leftMargin: 6; right: durationText.left; rightMargin: 8; verticalCenter: parent.verticalCenter }
                text: panel.utils.fileName(row.modelData)
                color: row.isCurrent ? panel.theme.accent : panel.theme.text
                font.pixelSize: 13
                elide: Text.ElideMiddle
            }
            Text {
                id: durationText
                anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
                text: row.duration >= 0 ? panel.utils.formatTime(row.duration) : ""
                color: row.isCurrent ? panel.theme.accent : panel.theme.muted
                font.pixelSize: 12
                font.family: "monospace"
            }
            MouseArea {
                id: rowArea

                property real pressY: 0
                property bool dragging: false

                // Position d'arrivée sous la souris (lignes de 30 px, pas de filtre actif)
                function targetIndex(mouseY) {
                    const y = mapToItem(list.contentItem, 0, mouseY).y
                    return Math.max(0, Math.min(panel.count - 1, Math.floor(y / 30)))
                }

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: dragging ? Qt.ClosedHandCursor : Qt.ArrowCursor
                onPressed: (mouse) => { pressY = mouse.y; dragging = false }
                onPositionChanged: (mouse) => {
                    // Réorganiser : seulement sans recherche (des lignes seraient masquées)
                    if (!pressed || panel.filter !== "")
                        return
                    if (!dragging && Math.abs(mouse.y - pressY) > 6)
                        dragging = true
                    if (dragging)
                        panel.dropIndex = targetIndex(mouse.y)
                }
                onReleased: {
                    if (dragging && panel.dropIndex >= 0 && panel.dropIndex !== row.index) {
                        panel.playlist.moveItem(row.index, panel.dropIndex)
                        panel.selected = panel.dropIndex
                    }
                    dragging = false
                    panel.dropIndex = -1
                }
                onClicked: if (!dragging) panel.selected = row.index
                onDoubleClicked: panel.activated(row.index)
            }
        }

        Text {
            anchors.centerIn: parent
            visible: panel.count === 0
            text: "Playlist vide"
            color: panel.theme.muted
            font.pixelSize: 13
        }
    }

    // ------------------------------------------------- Barre d'outils (bas)
    Item {
        id: toolbar

        // Recherche ouverte : le champ remplace les boutons
        property bool searching: false

        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 44

        Rectangle {
            anchors { left: parent.left; right: parent.right; top: parent.top }
            height: 1
            color: panel.theme.border
        }

        component ToolButton: Rectangle {
            id: tb

            property var theme
            property string label
            property string icon
            signal clicked()

            implicitWidth: icon !== "" ? 26 : Math.max(26, tbText.implicitWidth + 14)
            height: 24
            radius: 4
            opacity: enabled ? 1 : 0.35
            color: tbArea.pressed ? panel.theme.strong : tbArea.containsMouse ? panel.theme.raised : panel.theme.subtle

            Icon {
                anchors.centerIn: parent
                visible: tb.icon !== ""
                name: tb.icon
                size: 14
                strokeWidth: 2.2
                color: tb.theme.text
            }
            Text {
                id: tbText
                anchors.centerIn: parent
                visible: tb.icon === ""
                text: tb.label
                color: tb.theme.text
                font.pixelSize: 11
                font.weight: Font.DemiBold
            }
            MouseArea {
                id: tbArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: tb.clicked()
            }
        }

        Row {
            id: tools
            visible: !toolbar.searching
            anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
            spacing: 3

            readonly property bool hasSelection: panel.selected >= 0 && panel.selected < panel.count

            ToolButton { theme: panel.theme; icon: "arrow-up-to-line"; enabled: tools.hasSelection; onClicked: panel.moveSelected(0) }
            ToolButton { theme: panel.theme; icon: "chevron-up"; enabled: tools.hasSelection; onClicked: panel.moveSelected(panel.selected - 1) }
            ToolButton { theme: panel.theme; icon: "chevron-down"; enabled: tools.hasSelection; onClicked: panel.moveSelected(panel.selected + 1) }
            ToolButton { theme: panel.theme; icon: "arrow-down-to-line"; enabled: tools.hasSelection; onClicked: panel.moveSelected(panel.count - 1) }
            Item { width: 5; height: 1 }
            ToolButton { id: addButton; theme: panel.theme; label: "AJOUTER"; onClicked: panel.addRequested(addButton) }
            ToolButton { theme: panel.theme; label: "RETIRER"; enabled: tools.hasSelection; onClicked: panel.removeSelected() }
            ToolButton { theme: panel.theme; label: "TRIER"; enabled: panel.count > 1; onClicked: panel.playlist.sort() }
        }

        // Champ de recherche, ouvert par la loupe
        Rectangle {
            visible: toolbar.searching
            anchors { left: parent.left; leftMargin: 8; right: searchButton.left; rightMargin: 6; verticalCenter: parent.verticalCenter }
            height: 26
            radius: 4
            color: panel.theme.field
            border.color: search.activeFocus ? panel.theme.accent : panel.theme.border

            TextInput {
                id: search
                anchors { fill: parent; leftMargin: 8; rightMargin: 8 }
                verticalAlignment: TextInput.AlignVCenter
                color: panel.theme.text
                selectionColor: panel.theme.accent
                font.pixelSize: 12
                clip: true
                onTextChanged: panel.filter = text
                Keys.onEscapePressed: panel.closeSearch()
            }
            Text {
                anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
                visible: search.text === ""
                text: "Rechercher dans la playlist"
                color: panel.theme.muted
                font.pixelSize: 12
            }
        }

        // Loupe : ouvre la recherche ; ✕ : la ferme et l'efface
        ToolButton {
            id: searchButton
            anchors { right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter }
            theme: panel.theme
            icon: toolbar.searching ? "x" : "search"
            onClicked: {
                if (toolbar.searching) {
                    panel.closeSearch()
                } else {
                    toolbar.searching = true
                    search.forceActiveFocus()
                }
            }
        }
    }
}
