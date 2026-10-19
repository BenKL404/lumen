import QtQuick

// Menu déroulant avec un niveau de sous-menus. Chaque entrée est un objet :
//   { label, shortcut, icon, checked, enabled, action }   entrée simple (`icon` : nom d'une Icon)
//   { label, icon, submenu: [ …entrées… ] }               ouvre un sous-menu
//   { separator: true }
Item {
    id: menu

    property var theme
    property var entries: []
    property var subEntries: []

    // Ouvre le menu sous (ou au-dessus de) `anchor`
    function popup(anchor, above) {
        const p = anchor.mapToItem(parent, 0, above ? 0 : anchor.height)
        open(p.x, above ? p.y - main.height - 6 : p.y + 4)
    }

    // Ouvre le menu à une position (coordonnées du parent), sans sortir de la fenêtre
    function popupAt(x, y) {
        open(x, y)
    }

    function open(x, y) {
        closeSubmenu()
        main.x = Math.max(4, Math.min(x, parent.width - main.width - 4))
        main.y = Math.max(4, Math.min(y, parent.height - main.height - 4))
        visible = true
    }

    function close() {
        closeSubmenu()
        visible = false
    }

    function openSubmenu(entryItem, items) {
        subEntries = items
        const p = entryItem.mapToItem(menu, 0, 0)
        // À droite du menu, ou à gauche s'il n'y a pas la place
        const right = main.x + main.width - 4
        sub.x = right + sub.width <= parent.width - 4 ? right : main.x - sub.width + 4
        sub.y = Math.max(4, Math.min(p.y - 6, parent.height - sub.height - 4))
        sub.visible = true
    }

    function closeSubmenu() {
        sub.visible = false
        subEntries = []
    }

    visible: false
    anchors.fill: parent

    component Panel: Rectangle {
        id: panel

        property var theme
        property var model: []
        signal triggered(var entry, Item item)
        signal hoveredEntry(var entry, Item item)
        // Ligne survolée : une seule à la fois. Dans une liste qui défile, Qt ne signale pas
        // toujours la sortie de la souris, et plusieurs lignes resteraient surlignées.
        property Item hoveredItem: null
        onVisibleChanged: hoveredItem = null
        HoverHandler { onHoveredChanged: if (!hovered) panel.hoveredItem = null }

        width: 290
        // Calculée depuis les entrées (et non depuis la colonne) : exacte dès l'ouverture,
        // avant que les délégués soient créés, pour positionner le menu au bon endroit.
        // Limitée à la fenêtre : au-delà, les entrées défilent.
        readonly property real fullHeight: 12 + model.reduce((h, e) => h + (e.separator ? 9 : 30), 0)
        height: Math.min(fullHeight, menu.height - 8)
        radius: 10
        color: theme.menu
        border.color: theme.border

        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons } // absorbe les clics

        Flickable {
            id: scroller
            anchors { fill: parent; margins: 6 }
            contentHeight: column.height
            clip: true
            interactive: contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            // Le sous-menu suit sa ligne : le fermer quand la liste défile
            onContentYChanged: if (panel === main) menu.closeSubmenu()

        Column {
            id: column
            width: scroller.width

            Repeater {
                model: panel.model

                delegate: Item {
                    id: entry

                    required property var modelData
                    readonly property bool isSeparator: modelData.separator === true
                    readonly property bool isEnabled: modelData.enabled !== false
                    readonly property bool hasSubmenu: modelData.submenu !== undefined

                    width: column.width
                    height: isSeparator ? 9 : 30

                    Rectangle {
                        visible: entry.isSeparator
                        anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: 6 }
                        height: 1
                        color: panel.theme.border
                    }

                    Rectangle {
                        visible: !entry.isSeparator
                        anchors.fill: parent
                        radius: 6
                        opacity: entry.isEnabled ? 1 : 0.4
                        color: panel.hoveredItem === entry && entry.isEnabled ? panel.theme.surfaceHover : "transparent"

                        // Coche, ou icône de l'entrée
                        Icon {
                            anchors { left: parent.left; leftMargin: 9; verticalCenter: parent.verticalCenter }
                            name: entry.modelData.checked ? "check" : (entry.modelData.icon || "")
                            size: 15
                            strokeWidth: entry.modelData.checked ? 2.5 : 1.8
                            color: entry.modelData.checked ? panel.theme.accent : panel.theme.muted
                        }
                        Text {
                            anchors { left: parent.left; leftMargin: 34; right: rightText.left; rightMargin: 8; verticalCenter: parent.verticalCenter }
                            text: entry.modelData.label || ""
                            color: entry.modelData.checked ? panel.theme.accent : panel.theme.text
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                        Item {
                            id: rightText
                            anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
                            width: entry.hasSubmenu ? 14 : shortcutText.implicitWidth
                            height: 14

                            Text {
                                id: shortcutText
                                anchors { right: parent.right; verticalCenter: parent.verticalCenter }
                                visible: !entry.hasSubmenu
                                text: entry.modelData.shortcut || ""
                                color: panel.theme.muted
                                font.pixelSize: 12
                            }
                            Icon {
                                anchors.centerIn: parent
                                visible: entry.hasSubmenu
                                name: "chevron-right"
                                size: 14
                                color: panel.theme.muted
                            }
                        }
                        MouseArea {
                            id: entryArea
                            anchors.fill: parent
                            enabled: !entry.isSeparator && entry.isEnabled
                            hoverEnabled: true
                            onContainsMouseChanged: {
                                if (containsMouse) {
                                    panel.hoveredItem = entry
                                    panel.hoveredEntry(entry.modelData, entry)
                                } else if (panel.hoveredItem === entry) {
                                    panel.hoveredItem = null
                                }
                            }
                            onClicked: panel.triggered(entry.modelData, entry)
                        }
                    }
                }
            }
        }
        }
    }

    Panel {
        id: main
        theme: menu.theme
        model: menu.entries
        onHoveredEntry: (entry, item) => {
            if (entry.submenu)
                submenuTimer.request(entry, item)
            else
                submenuTimer.request(null, null)
        }
        onTriggered: (entry, item) => {
            if (entry.submenu) {
                submenuTimer.stop()
                menu.openSubmenu(item, entry.submenu)
                return
            }
            menu.close()
            entry.action()
        }
    }

    Panel {
        id: sub
        visible: false
        theme: menu.theme
        model: menu.subEntries
        onTriggered: (entry) => {
            menu.close()
            entry.action()
        }
    }

    // Ouvre (ou ferme) le sous-menu après un court survol, pour ne pas clignoter
    // quand la souris traverse le menu
    Timer {
        id: submenuTimer

        property var entry: null
        property Item item: null

        function request(entry, item) {
            this.entry = entry
            this.item = item
            restart()
        }

        interval: 150
        onTriggered: {
            if (entry)
                menu.openSubmenu(item, entry.submenu)
            else
                menu.closeSubmenu()
        }
    }
}
