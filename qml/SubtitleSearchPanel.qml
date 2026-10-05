import QtQuick

// Recherche de sous-titres sur OpenSubtitles (Ctrl+J) : titre et langues, liste des
// résultats (correspondances exactes avec le fichier en tête), un clic télécharge et active.
Rectangle {
    id: panel

    property var theme
    property var search      // SubtitleSearch (online_subtitles.rs)
    property bool hasKey: false
    property string mediaUrl: ""

    signal closeRequested()
    signal preferencesRequested()
    signal searchRequested(string query, string languages)
    signal downloadRequested(var result)

    property var results: []
    property string status: ""

    function open(query, languages) {
        queryField.text = query
        languageField.text = languages
        results = []
        status = ""
        visible = true
        if (hasKey && query !== "")
            submit()
        else
            queryField.forceActiveFocus()
    }

    function submit() {
        if (queryField.text.trim() === "" && mediaUrl === "")
            return
        results = []
        status = "Recherche…"
        searchRequested(queryField.text, languageField.text)
    }

    function showResults(json) {
        results = JSON.parse(json)
        status = results.length === 0 ? "Aucun sous-titre trouvé : essaie un autre titre ou une autre langue"
               : results.length + " résultat" + (results.length > 1 ? "s" : "")
    }

    width: 640
    height: 500
    radius: theme.radius
    color: theme.menu
    border.color: theme.border

    // Absorbe les clics : ne pas fermer en cliquant dans la fenêtre
    MouseArea { anchors.fill: parent }

    Text {
        id: title
        anchors { left: parent.left; top: parent.top; leftMargin: 20; topMargin: 18 }
        text: "Rechercher des sous-titres"
        color: panel.theme.text
        font.pixelSize: 16
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

    component Field: Rectangle {
        id: field
        property alias text: input.text
        property alias input: input
        property string placeholder
        signal accepted()

        height: 34
        radius: 6
        color: panel.theme.field
        border.color: input.activeFocus ? panel.theme.accent : panel.theme.border

        TextInput {
            id: input
            anchors { fill: parent; leftMargin: 10; rightMargin: 10 }
            verticalAlignment: TextInput.AlignVCenter
            color: panel.theme.text
            selectionColor: panel.theme.accent
            font.pixelSize: 13
            clip: true
            onAccepted: field.accepted()
            Keys.onEscapePressed: panel.closeRequested()
        }
        Text {
            anchors { left: parent.left; leftMargin: 10; verticalCenter: parent.verticalCenter }
            visible: input.text === ""
            text: field.placeholder
            color: panel.theme.muted
            font.pixelSize: 13
        }
    }

    // ------------------------------------------------------------ Recherche
    Row {
        id: searchRow
        anchors { left: parent.left; right: parent.right; top: title.bottom; leftMargin: 20; rightMargin: 20; topMargin: 16 }
        spacing: 8
        visible: panel.hasKey

        Field {
            id: queryField
            width: parent.width - languageField.width - searchButton.width - 16
            placeholder: "Titre du film ou de la série"
            onAccepted: panel.submit()
        }
        Field {
            id: languageField
            width: 90
            placeholder: "fr,en"
            onAccepted: panel.submit()
        }
        Rectangle {
            id: searchButton
            width: searchLabel.implicitWidth + 28
            height: 34
            radius: 6
            color: searchArea.containsMouse ? Qt.lighter(panel.theme.accent, 1.1) : panel.theme.accent
            opacity: panel.search.busy ? 0.5 : 1
            Text {
                id: searchLabel
                anchors.centerIn: parent
                text: "Rechercher"
                color: panel.theme.onAccent
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            MouseArea {
                id: searchArea
                anchors.fill: parent
                enabled: !panel.search.busy
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.submit()
            }
        }
    }

    Text {
        id: statusText
        anchors { left: parent.left; right: parent.right; top: searchRow.bottom; leftMargin: 20; rightMargin: 20; topMargin: 10 }
        visible: panel.hasKey
        text: panel.status
        color: panel.theme.muted
        font.pixelSize: 12
        elide: Text.ElideRight
    }

    // ------------------------------------------------------------ Résultats
    ListView {
        anchors { left: parent.left; right: parent.right; top: statusText.bottom; bottom: parent.bottom; margins: 12; topMargin: 8 }
        visible: panel.hasKey
        clip: true
        spacing: 2
        model: panel.results
        boundsBehavior: Flickable.StopAtBounds

        delegate: Rectangle {
            id: result
            required property var modelData
            width: ListView.view.width
            height: 44
            radius: 8
            color: resultArea.containsMouse ? panel.theme.surfaceHover : "transparent"

            Rectangle {
                id: languageBadge
                anchors { left: parent.left; leftMargin: 10; verticalCenter: parent.verticalCenter }
                width: 34
                height: 20
                radius: 4
                color: "transparent"
                border.color: panel.theme.border
                Text {
                    anchors.centerIn: parent
                    text: result.modelData.language.toUpperCase()
                    color: panel.theme.text
                    font.pixelSize: 10
                    font.weight: Font.DemiBold
                }
            }
            Column {
                anchors { left: languageBadge.right; leftMargin: 12; right: downloadsText.left; rightMargin: 12; verticalCenter: parent.verticalCenter }
                spacing: 2
                Text {
                    width: parent.width
                    text: result.modelData.release || "Sans nom"
                    color: panel.theme.text
                    font.pixelSize: 13
                    elide: Text.ElideMiddle
                }
                Text {
                    width: parent.width
                    visible: text !== ""
                    text: [result.modelData.exact ? "Correspond exactement à ce fichier" : "",
                           result.modelData.hearingImpaired ? "Pour malentendants" : ""].filter(t => t).join(" · ")
                    color: result.modelData.exact ? panel.theme.accent : panel.theme.muted
                    font.pixelSize: 11
                }
            }
            Text {
                id: downloadsText
                anchors { right: parent.right; rightMargin: 12; verticalCenter: parent.verticalCenter }
                text: result.modelData.downloads.toLocaleString(Qt.locale("fr_FR"), "f", 0) + " ↓"
                color: panel.theme.muted
                font.pixelSize: 11
            }
            MouseArea {
                id: resultArea
                anchors.fill: parent
                enabled: !panel.search.busy
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    panel.status = "Téléchargement…"
                    panel.downloadRequested(result.modelData)
                }
            }
        }
    }

    // ------------------------------------------------------- Sans clé d'API
    Column {
        anchors { left: parent.left; right: parent.right; top: title.bottom; leftMargin: 20; rightMargin: 20; topMargin: 24 }
        visible: !panel.hasKey
        spacing: 14

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: "La recherche utilise OpenSubtitles, qui demande une clé d'API gratuite :\n\n"
                  + "1. Crée un compte gratuit sur opensubtitles.com\n"
                  + "2. Dans ton profil, ouvre « API consumers » et crée une clé\n"
                  + "3. Colle-la dans Préférences › Sous-titres"
            color: panel.theme.text
            font.pixelSize: 13
            lineHeight: 1.3
        }
        Row {
            spacing: 8
            Repeater {
                model: [
                    { label: "Créer une clé", action: () => Qt.openUrlExternally("https://www.opensubtitles.com/consumers") },
                    { label: "Préférences", action: () => panel.preferencesRequested() }
                ]
                delegate: Rectangle {
                    required property var modelData
                    width: buttonText.implicitWidth + 24
                    height: 32
                    radius: 6
                    color: buttonArea.containsMouse ? panel.theme.surfaceHover : panel.theme.subtle
                    border.color: panel.theme.border
                    Text {
                        id: buttonText
                        anchors.centerIn: parent
                        text: parent.modelData.label
                        color: panel.theme.text
                        font.pixelSize: 12
                    }
                    MouseArea {
                        id: buttonArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: parent.modelData.action()
                    }
                }
            }
        }
    }
}
