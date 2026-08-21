import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: root
    property var task
    property bool isRunning: false
    signal toggleClicked()
    signal logsClicked()
    signal editClicked()
    signal moveUpClicked()
    signal moveDownClicked()
    signal deleteClicked()

    height: 60
    color: "#2b2b2b"
    radius: 6
    border.color: isRunning ? "#2ecc71" : "#444444"
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 12

        Rectangle {
            width: 12
            height: 12
            radius: 6
            color: root.isRunning ? "#2ecc71" : "#e74c3c"
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: root.task ? root.task.name : ""
                color: "#ffffff"
                font.bold: true
                font.pixelSize: 14
            }

            Text {
                text: root.task ? root.task.command + " (" + root.task.working_directory + ")" : ""
                color: "#888888"
                font.pixelSize: 11
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
        }

        Button {
            text: root.isRunning ? "Stop" : "Start"
            onClicked: root.toggleClicked()
        }

        Button {
            text: "Logs"
            onClicked: root.logsClicked()
        }

        Button {
            text: "↑"
            onClicked: root.moveUpClicked()
        }

        Button {
            text: "↓"
            onClicked: root.moveDownClicked()
        }

        Button {
            text: "Edit"
            onClicked: root.editClicked()
        }

        Button {
            text: "Delete"
            onClicked: root.deleteClicked()
        }
    }
}
