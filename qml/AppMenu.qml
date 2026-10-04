import QtQuick

// Menu déroulant générique : chaque entrée est { label, shortcut, checked, action }
// ou { separator: true }
Rectangle {
    id: menu

    property var theme
    property var entries: []

    // Ouvre le menu sous (ou au-dessus de) `anchor`, sans sortir de la fenêtre
    function popup(anchor, above) {
        const host = menu.parent
        const p = anchor.mapToItem(host, 0, above ? 0 : anchor.height)
        x = Math.max(4, Math.min(p.x, host.width - width - 4))
        y = above ? Math.max(4, p.y - height - 6) : p.y + 4
        visible = true
    }

    visible: false
    width: 280
    height: column.implicitHeight + 12
    radius: 10
    color: theme.menu
    border.color: theme.border

    MouseArea { anchors.fill: parent } // absorbe les clics

    Column {
        id: column
        anchors { left: parent.left; right: parent.right; top: parent.top; margins: 6 }

        Repeater {
            model: menu.entries

            delegate: Item {
                id: entry

                required property var modelData

                width: column.width
                height: modelData.separator ? 9 : 30

                Rectangle {
                    visible: entry.modelData.separator === true
                    anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: 6 }
                    height: 1
                    color: menu.theme.border
                }

                Rectangle {
                    visible: !entry.modelData.separator
                    anchors.fill: parent
                    radius: 6
                    color: entryArea.containsMouse ? menu.theme.surfaceHover : "transparent"

                    Text {
                        anchors { left: parent.left; leftMargin: 10; verticalCenter: parent.verticalCenter }
                        width: 14
                        text: entry.modelData.checked ? "✓" : ""
                        color: menu.theme.accent
                        font.pixelSize: 13
                    }
                    Text {
                        anchors { left: parent.left; leftMargin: 30; verticalCenter: parent.verticalCenter }
                        text: entry.modelData.label || ""
                        color: menu.theme.text
                        font.pixelSize: 13
                    }
                    Text {
                        anchors { right: parent.right; rightMargin: 10; verticalCenter: parent.verticalCenter }
                        text: entry.modelData.shortcut || ""
                        color: menu.theme.muted
                        font.pixelSize: 12
                    }
                    MouseArea {
                        id: entryArea
                        anchors.fill: parent
                        enabled: !entry.modelData.separator
                        hoverEnabled: true
                        onClicked: {
                            menu.visible = false
                            entry.modelData.action()
                        }
                    }
                }
            }
        }
    }
}
