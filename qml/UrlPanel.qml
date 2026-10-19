import QtQuick

// « Ouvrir une URL » (Ctrl+U) : lien YouTube ou de tout site pris en charge par yt-dlp,
// adresse directe d'un flux vidéo, liste de chaînes IPTV (lien M3U), ou compte IPTV
// Xtream Codes (serveur, utilisateur, mot de passe).
Rectangle {
    id: panel

    property var theme

    signal accepted(string url)
    signal xtreamRequested(string server, string user, string password)
    signal closeRequested()

    readonly property bool xtreamReady: server.text.trim() !== "" && user.text.trim() !== "" && password.text !== ""

    function submitXtream() {
        if (xtreamReady)
            xtreamRequested(server.text, user.text, password.text)
    }

    // Champ de saisie (thème passé explicitement : un composant en ligne ne voit pas `panel`)
    component Field: Rectangle {
        id: fieldBox
        property var theme
        property string placeholder
        property alias text: input.text
        property alias echoMode: input.echoMode
        signal submitted()
        function takeFocus() { input.forceActiveFocus() }
        height: 36
        radius: 6
        color: theme.field
        border.color: input.activeFocus ? theme.accent : theme.border

        TextInput {
            id: input
            anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
            verticalAlignment: TextInput.AlignVCenter
            color: fieldBox.theme.text
            selectionColor: fieldBox.theme.accent
            font.pixelSize: 13
            clip: true
            onAccepted: fieldBox.submitted()
        }
        Text {
            anchors { left: parent.left; leftMargin: 12; right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
            visible: input.text === ""
            text: fieldBox.placeholder
            color: fieldBox.theme.muted
            font.pixelSize: 13
            elide: Text.ElideRight
        }
    }

    function open(initial) {
        field.text = initial || ""
        visible = true
        field.forceActiveFocus()
        field.selectAll()
    }

    function submit() {
        const url = field.text.trim()
        if (url !== "")
            accepted(url)
    }

    width: 560
    height: 316
    radius: theme.radius
    // Opaque, même couleur que la playlist
    color: theme.chrome
    border.color: theme.border

    // Absorbe les clics : ne pas mettre en pause en cliquant dans la fenêtre
    MouseArea { anchors.fill: parent }

    Text {
        id: title
        anchors { left: parent.left; top: parent.top; leftMargin: 20; topMargin: 18 }
        text: "Ouvrir une vidéo en ligne"
        color: panel.theme.text
        font.pixelSize: 15
        font.weight: Font.DemiBold
    }
    IconButton {
        anchors { right: parent.right; rightMargin: 10; verticalCenter: title.verticalCenter }
        implicitHeight: 32
        theme: panel.theme
        icon: "x"
        iconSize: 15
        onClicked: panel.closeRequested()
    }

    Rectangle {
        id: box
        anchors { left: parent.left; right: openButton.left; top: title.bottom; leftMargin: 20; rightMargin: 10; topMargin: 16 }
        height: 36
        radius: 6
        color: panel.theme.field
        border.color: field.activeFocus ? panel.theme.accent : panel.theme.border

        TextInput {
            id: field
            anchors { fill: parent; leftMargin: 12; rightMargin: 12 }
            verticalAlignment: TextInput.AlignVCenter
            color: panel.theme.text
            selectionColor: panel.theme.accent
            font.pixelSize: 13
            clip: true
            onAccepted: panel.submit()
            Keys.onEscapePressed: panel.closeRequested()
        }
        Text {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
            visible: field.text === ""
            text: "https://www.youtube.com/watch?v=…"
            color: panel.theme.muted
            font.pixelSize: 13
        }
    }

    Rectangle {
        id: openButton
        anchors { right: parent.right; rightMargin: 20; verticalCenter: box.verticalCenter }
        width: openLabel.implicitWidth + 28
        height: 36
        radius: 6
        color: openArea.containsMouse ? Qt.lighter(panel.theme.accent, 1.1) : panel.theme.accent
        opacity: field.text.trim() !== "" ? 1 : 0.5

        Text {
            id: openLabel
            anchors.centerIn: parent
            text: "Ouvrir"
            color: panel.theme.onAccent
            font.pixelSize: 13
            font.weight: Font.DemiBold
        }
        MouseArea {
            id: openArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: panel.submit()
        }
    }

    Text {
        anchors { left: parent.left; right: parent.right; top: box.bottom; leftMargin: 20; rightMargin: 20; topMargin: 12 }
        id: hint
        text: "YouTube, Vimeo, Dailymotion… (via yt-dlp), flux direct ou liste de chaînes IPTV (.m3u)"
        color: panel.theme.muted
        font.pixelSize: 12
        elide: Text.ElideRight
    }

    // ------------------------------------------------------------ Compte IPTV
    Rectangle {
        id: divider
        anchors { left: parent.left; right: parent.right; top: hint.bottom; topMargin: 18 }
        height: 1
        color: panel.theme.divider
    }
    Text {
        id: xtreamTitle
        anchors { left: parent.left; top: divider.bottom; leftMargin: 20; topMargin: 16 }
        text: "Compte IPTV (Xtream Codes)"
        color: panel.theme.text
        font.pixelSize: 14
        font.weight: Font.DemiBold
    }
    Field {
        id: server
        anchors { left: parent.left; right: parent.right; top: xtreamTitle.bottom; leftMargin: 20; rightMargin: 20; topMargin: 12 }
        theme: panel.theme
        placeholder: "Serveur, par exemple http://exemple.tv:8080"
        onSubmitted: user.takeFocus()
    }
    Row {
        id: credentials
        anchors { left: parent.left; right: parent.right; top: server.bottom; leftMargin: 20; rightMargin: 20; topMargin: 10 }
        spacing: 10
        readonly property real fieldWidth: (width - loadButton.width - 2 * spacing) / 2

        Field {
            id: user
            width: credentials.fieldWidth
            theme: panel.theme
            placeholder: "Utilisateur"
            onSubmitted: password.takeFocus()
        }
        Field {
            id: password
            width: credentials.fieldWidth
            theme: panel.theme
            placeholder: "Mot de passe"
            echoMode: TextInput.Password
            onSubmitted: panel.submitXtream()
        }
        Rectangle {
            id: loadButton
            width: loadLabel.implicitWidth + 28
            height: 36
            radius: 6
            color: loadArea.containsMouse ? Qt.lighter(panel.theme.accent, 1.1) : panel.theme.accent
            opacity: panel.xtreamReady ? 1 : 0.5

            Text {
                id: loadLabel
                anchors.centerIn: parent
                text: "Charger"
                color: panel.theme.onAccent
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            MouseArea {
                id: loadArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.submitXtream()
            }
        }
    }
    Text {
        anchors { left: parent.left; right: parent.right; top: credentials.bottom; leftMargin: 20; rightMargin: 20; topMargin: 12 }
        text: "Identifiants fournis par ton abonnement ; la liste est retenue (menu › Chaînes IPTV)"
        color: panel.theme.muted
        font.pixelSize: 12
        elide: Text.ElideRight
    }
}
