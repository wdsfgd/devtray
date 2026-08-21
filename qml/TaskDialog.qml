import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Popup {
    id: root

    property string taskId: ""
    property alias taskName: nameField.text
    property alias command: cmdField.text
    property alias workingDir: dirField.text
    property alias group: groupField.text

    signal saved(string id, string name, string command, string workingDir, string group)

    modal: true
    focus: true
    dim: true
    closePolicy: Popup.CloseOnEscape

    width: 360
    implicitHeight: mainLayout.implicitHeight + 28
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)

    onOpened: {
        nameField.forceActiveFocus()
    }

    background: Rectangle {
        color: "#242424"
        border.color: "#3d3d3d"
        border.width: 1
        radius: 8
    }

    function submit() {
        if (nameField.text.trim() === "" || cmdField.text.trim() === "") {
            return
        }
        root.saved(root.taskId, nameField.text.trim(), cmdField.text.trim(), dirField.text.trim(), groupField.text.trim())
        root.close()
    }

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: 14
        spacing: 10

        // Dialog Header
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: root.taskId === "" ? "Add Task" : "Edit Task"
                color: "#ffffff"
                font.bold: true
                font.pixelSize: 14
                Layout.fillWidth: true
            }

            Rectangle {
                width: 22
                height: 22
                radius: 11
                color: closeArea.containsMouse ? "#3a3a3a" : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "✕"
                    color: "#999999"
                    font.pixelSize: 11
                }

                MouseArea {
                    id: closeArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.close()
                }
            }
        }

        // Form Fields
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 8

            // Task Name
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 3

                Text {
                    text: "Task Name:"
                    color: "#dcdcdc"
                    font.pixelSize: 12
                }

                TextField {
                    id: nameField
                    Layout.fillWidth: true
                    implicitHeight: 32
                    placeholderText: "e.g. Frontend"
                    color: "#ffffff"
                    placeholderTextColor: "#666666"
                    selectByMouse: true
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    font.pixelSize: 12
                    leftPadding: 8
                    rightPadding: 8

                    background: Rectangle {
                        color: "#1c1c1c"
                        border.color: nameField.activeFocus ? "#3584e4" : "#3d3d3d"
                        border.width: nameField.activeFocus ? 2 : 1
                        radius: 4
                    }

                    onAccepted: cmdField.forceActiveFocus()
                }
            }

            // Command
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 3

                Text {
                    text: "Command:"
                    color: "#dcdcdc"
                    font.pixelSize: 12
                }

                TextField {
                    id: cmdField
                    Layout.fillWidth: true
                    implicitHeight: 32
                    placeholderText: "e.g. npm run dev"
                    color: "#ffffff"
                    placeholderTextColor: "#666666"
                    selectByMouse: true
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    font.pixelSize: 12
                    leftPadding: 8
                    rightPadding: 8

                    background: Rectangle {
                        color: "#1c1c1c"
                        border.color: cmdField.activeFocus ? "#3584e4" : "#3d3d3d"
                        border.width: cmdField.activeFocus ? 2 : 1
                        radius: 4
                    }

                    onAccepted: dirField.forceActiveFocus()
                }
            }

            // Working Directory
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 3

                Text {
                    text: "Working Directory:"
                    color: "#dcdcdc"
                    font.pixelSize: 12
                }

                TextField {
                    id: dirField
                    Layout.fillWidth: true
                    implicitHeight: 32
                    text: "."
                    color: "#ffffff"
                    placeholderTextColor: "#666666"
                    selectByMouse: true
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    font.pixelSize: 12
                    leftPadding: 8
                    rightPadding: 8

                    background: Rectangle {
                        color: "#1c1c1c"
                        border.color: dirField.activeFocus ? "#3584e4" : "#3d3d3d"
                        border.width: dirField.activeFocus ? 2 : 1
                        radius: 4
                    }

                    onAccepted: groupField.forceActiveFocus()
                }
            }

            // Group
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 3

                Text {
                    text: "Group (Optional):"
                    color: "#dcdcdc"
                    font.pixelSize: 12
                }

                TextField {
                    id: groupField
                    Layout.fillWidth: true
                    implicitHeight: 32
                    placeholderText: "e.g. Web"
                    color: "#ffffff"
                    placeholderTextColor: "#666666"
                    selectByMouse: true
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    font.pixelSize: 12
                    leftPadding: 8
                    rightPadding: 8

                    background: Rectangle {
                        color: "#1c1c1c"
                        border.color: groupField.activeFocus ? "#3584e4" : "#3d3d3d"
                        border.width: groupField.activeFocus ? 2 : 1
                        radius: 4
                    }

                    onAccepted: root.submit()
                }
            }
        }

        Item {
            Layout.preferredHeight: 4
        }

        // Action Buttons (Right-aligned Cancel & Save)
        RowLayout {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignRight
            spacing: 8

            Item {
                Layout.fillWidth: true
            }

            Button {
                id: cancelBtn
                text: "Cancel"
                implicitHeight: 30
                implicitWidth: 72

                background: Rectangle {
                    color: cancelBtn.down ? "#1f1f1f" : (cancelBtn.hovered ? "#383838" : "#2c2c2c")
                    border.color: cancelBtn.activeFocus ? "#3584e4" : "#444444"
                    border.width: cancelBtn.activeFocus ? 2 : 1
                    radius: 4
                }

                contentItem: Text {
                    text: cancelBtn.text
                    color: "#dcdcdc"
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.close()
            }

            Button {
                id: saveBtn
                text: "Save"
                implicitHeight: 30
                implicitWidth: 72

                background: Rectangle {
                    color: saveBtn.down ? "#1c60b3" : (saveBtn.hovered ? "#2b7de0" : "#3584e4")
                    border.color: "#3584e4"
                    border.width: 1
                    radius: 4
                }

                contentItem: Text {
                    text: saveBtn.text
                    color: "#ffffff"
                    font.bold: true
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.submit()
            }
        }
    }
}
