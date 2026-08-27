import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root

    property var task
    property bool isRunning: false
    property var listView: null

    signal toggleClicked()
    signal logsClicked()
    signal editClicked()
    signal deleteClicked()
    signal reorderRequested(string taskId, int targetIndex)

    height: 38

    Rectangle {
        id: cardContainer
        anchors.left: parent.left
        anchors.right: parent.right
        height: 38
        y: 0
        z: dragArea.drag.active ? 100 : 1

        color: dragArea.drag.active ? "#28323e" : (rowMouseArea.containsMouse ? "#2a2a2a" : "transparent")
        radius: 4
        border.color: dragArea.drag.active ? "#3584e4" : (rowMouseArea.containsMouse ? "#3a3a3a" : "transparent")
        border.width: dragArea.drag.active ? 2 : 1
        opacity: dragArea.drag.active ? 0.95 : 1.0
        scale: dragArea.drag.active ? 1.02 : 1.0

        Behavior on scale {
            NumberAnimation { duration: 100 }
        }

        MouseArea {
            id: rowMouseArea
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 6
            anchors.rightMargin: 6
            spacing: 6

            // Drag Grip Handle
            Item {
                id: dragHandle
                implicitWidth: 16
                implicitHeight: 24
                Layout.preferredWidth: 16
                Layout.preferredHeight: 24
                Layout.alignment: Qt.AlignVCenter

                Text {
                    anchors.centerIn: parent
                    text: "⠿"
                    color: dragArea.drag.active ? "#3584e4" : (dragArea.containsMouse ? "#ffffff" : "#666666")
                    font.pixelSize: 14
                }

                ToolTip.visible: dragArea.containsMouse && !dragArea.drag.active
                ToolTip.text: "Drag to reorder"
                ToolTip.delay: 350

                MouseArea {
                    id: dragArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: drag.active ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                    drag.target: cardContainer
                    drag.axis: Drag.YAxis

                    function calculateTargetIndex(yPos) {
                        if (!root.listView || root.listView.count === 0) return -1
                        var idx = root.listView.indexAt(root.listView.width / 2, yPos)
                        if (idx >= 0) return idx

                        // Probe adjacent offsets if dropped on section header or item margin
                        var deltas = [5, -5, 10, -10, 15, -15, 20, -20, 25, -25, 30, -30]
                        for (var i = 0; i < deltas.length; i++) {
                            var probed = root.listView.indexAt(root.listView.width / 2, yPos + deltas[i])
                            if (probed >= 0) return probed
                        }

                        if (yPos <= 30) return 0
                        if (yPos >= root.listView.contentHeight - 30) return root.listView.count - 1
                        return -1
                    }

                    onReleased: {
                        if (root.task && root.task.id && root.listView) {
                            var centerPt = cardContainer.mapToItem(root.listView.contentItem, cardContainer.width / 2, cardContainer.height / 2)
                            var targetIdx = calculateTargetIndex(centerPt.y)
                            cardContainer.y = 0
                            if (targetIdx >= 0 && targetIdx !== model.index) {
                                root.reorderRequested(root.task.id, targetIdx)
                            }
                        } else {
                            cardContainer.y = 0
                        }
                    }
                }
            }

            // Status Indicator (Green/Red dot)
            Rectangle {
                id: statusDot
                implicitWidth: 10
                implicitHeight: 10
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
                    opacity: root.isRunning ? 0.3 : 0
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
                    implicitWidth: 36
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

                // Delete Button
                Button {
                    id: btnDelete
                    implicitWidth: 46
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
}
