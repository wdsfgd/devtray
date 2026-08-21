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

    height: 38
    color: rowMouseArea.containsMouse ? "#2a2a2a" : "transparent"
    radius: 4
    border.color: rowMouseArea.containsMouse ? "#3a3a3a" : "transparent"
    border.width: 1

    MouseArea {
        id: rowMouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        spacing: 6

        // Status Indicator (Green/Red dot)
        Rectangle {
            id: statusDot
            width: 10
            height: 10
            radius: 5
            color: root.isRunning ? "#2ecc71" : "#e74c3c"
            Layout.alignment: Qt.AlignVCenter
            Layout.preferredWidth: 10
            Layout.preferredHeight: 10

            // Subtle glow when running
            Rectangle {
                anchors.centerIn: parent
                width: 14
                height: 14
                radius: 7
                color: "#2ecc71"
                opacity: root.isRunning ? 0.25 : 0
                visible: root.isRunning
            }
        }

        // Task Name
        Text {
            id: nameLabel
            text: root.task ? root.task.name : ""
            color: "#ffffff"
            font.pixelSize: 13
            font.bold: true
            elide: Text.ElideRight
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            ToolTip.visible: nameHoverArea.containsMouse && (nameLabel.truncated || (root.task && root.task.command))
            ToolTip.text: root.task ? (root.task.name + "\n" + root.task.command + " (" + root.task.working_directory + ")") : ""
            ToolTip.delay: 400

            MouseArea {
                id: nameHoverArea
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.NoButton
            }
        }

        // Button Group (Compact GTK/Adwaita Dark Styled Buttons)
        RowLayout {
            spacing: 4
            Layout.alignment: Qt.AlignVCenter

            // Toggle Play / Stop Button
            Button {
                id: btnToggle
                implicitWidth: 28
                implicitHeight: 26

                ToolTip.visible: btnToggle.hovered
                ToolTip.text: root.isRunning ? "Stop Task" : "Start Task"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnToggle.down ? "#1f1f1f" : (btnToggle.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnToggle.hovered ? "#555555" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: root.isRunning ? "⏹" : "▶"
                    color: root.isRunning ? "#ff5555" : "#2ecc71"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.toggleClicked()
            }

            // Logs Button
            Button {
                id: btnLogs
                implicitWidth: 38
                implicitHeight: 26
                text: "Logs"

                ToolTip.visible: btnLogs.hovered
                ToolTip.text: "View Live Logs"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnLogs.down ? "#1f1f1f" : (btnLogs.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnLogs.hovered ? "#555555" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: btnLogs.text
                    color: "#dcdcdc"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.logsClicked()
            }

            // Edit Button
            Button {
                id: btnEdit
                implicitWidth: 38
                implicitHeight: 26
                text: "Edit"

                ToolTip.visible: btnEdit.hovered
                ToolTip.text: "Edit Task"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnEdit.down ? "#1f1f1f" : (btnEdit.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnEdit.hovered ? "#555555" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: btnEdit.text
                    color: "#dcdcdc"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.editClicked()
            }

            // Move Up Button
            Button {
                id: btnUp
                implicitWidth: 24
                implicitHeight: 26
                text: "↑"

                ToolTip.visible: btnUp.hovered
                ToolTip.text: "Move Up"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnUp.down ? "#1f1f1f" : (btnUp.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnUp.hovered ? "#555555" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: btnUp.text
                    color: "#dcdcdc"
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.moveUpClicked()
            }

            // Move Down Button
            Button {
                id: btnDown
                implicitWidth: 24
                implicitHeight: 26
                text: "↓"

                ToolTip.visible: btnDown.hovered
                ToolTip.text: "Move Down"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnDown.down ? "#1f1f1f" : (btnDown.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnDown.hovered ? "#555555" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: btnDown.text
                    color: "#dcdcdc"
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.moveDownClicked()
            }

            // Delete Button
            Button {
                id: btnDelete
                implicitWidth: 48
                implicitHeight: 26
                text: "Delete"

                ToolTip.visible: btnDelete.hovered
                ToolTip.text: "Delete Task"
                ToolTip.delay: 500

                background: Rectangle {
                    color: btnDelete.down ? "#1f1f1f" : (btnDelete.hovered ? "#4d2222" : "#2e2e2e")
                    border.color: btnDelete.hovered ? "#884444" : "#424242"
                    radius: 4
                }

                contentItem: Text {
                    text: btnDelete.text
                    color: btnDelete.hovered ? "#ff7777" : "#dcdcdc"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.deleteClicked()
            }
        }
    }
}
