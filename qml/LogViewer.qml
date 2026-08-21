import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Dialog {
    id: root
    property string taskName: ""
    property alias logContent: logTextArea.text

    title: "Live Logs: " + taskName
    width: 650
    height: 450
    modal: true
    standardButtons: Dialog.Close

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true

            TextArea {
                id: logTextArea
                readOnly: true
                font.family: "Monospace"
                font.pixelSize: 12
                color: "#00ff66"
                background: Rectangle { color: "#1e1e1e" }
                wrapMode: TextArea.Wrap
            }
        }

        RowLayout {
            Button {
                text: "Clear"
                onClicked: logTextArea.text = ""
            }
        }
    }
}
