import QtQuick

// Menu déroulant avec un niveau de sous-menus. Chaque entrée est un objet :
//   { label, shortcut, icon, checked, enabled, action }   entrée simple
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

        width: 290
        height: column.implicitHeight + 12
        radius: 10
        color: theme.menu
        border.color: theme.border

        MouseArea { anchors.fill: parent; acceptedButtons: Qt.AllButtons } // absorbe les clics

        Column {
            id: column
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 6 }

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
                        color: entryArea.containsMouse && entry.isEnabled ? panel.theme.surfaceHover : "transparent"

                        // Coche, ou icône de l'entrée
                        Text {
                            anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
                            width: 18
                            horizontalAlignment: Text.AlignHCenter
                            text: entry.modelData.checked ? "✓" : (entry.modelData.icon || "")
                            color: entry.modelData.checked ? panel.theme.accent : panel.theme.muted
                            font.pixelSize: 13
                        }
                        Text {
                            anchors { left: parent.left; leftMargin: 34; right: rightText.left; rightMargin: 8; verticalCenter: parent.verticalCenter }
                            text: entry.modelData.label || ""
                            color: entry.modelData.checked ? panel.theme.accent : panel.theme.text
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                        Text {
                            id: rightText
                            anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
                            text: entry.hasSubmenu ? "▸" : (entry.modelData.shortcut || "")
                            color: panel.theme.muted
                            font.pixelSize: 12
                        }
                        MouseArea {
                            id: entryArea
                            anchors.fill: parent
                            enabled: !entry.isSeparator && entry.isEnabled
                            hoverEnabled: true
                            onContainsMouseChanged: if (containsMouse) panel.hoveredEntry(entry.modelData, entry)
                            onClicked: panel.triggered(entry.modelData, entry)
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
