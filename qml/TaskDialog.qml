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
    property string errorText: ""
    property bool nameError: false
    property bool cmdError: false

    signal saved(string id, string name, string command, string workingDir, string group)

    modal: true
    focus: true
    dim: true
    closePolicy: Popup.CloseOnEscape
    padding: 0

    width: Math.min(parent ? parent.width - 24 : 360, 360)
    implicitHeight: mainLayout.implicitHeight + 28
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    onOpened: {
        root.errorText = ""
        root.nameError = false
        root.cmdError = false
        nameField.forceActiveFocus()
        nameField.selectAll()
    }

    background: Rectangle {
        color: "#242424"
        border.color: "#3d3d3d"
        border.width: 1
        radius: 8
    }

    function submit() {
        var nameTrimmed = nameField.text.trim()
        var cmdTrimmed = cmdField.text.trim()

        if (nameTrimmed === "") {
            root.nameError = true
            root.errorText = "Task name is required."
            nameField.forceActiveFocus()
            return
        }
        if (cmdTrimmed === "") {
            root.cmdError = true
            root.errorText = "Command is required."
            cmdField.forceActiveFocus()
            return
        }

        root.saved(root.taskId, nameTrimmed, cmdTrimmed, dirField.text.trim(), groupField.text.trim())
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
                implicitWidth: 22
                implicitHeight: 22
                Layout.preferredWidth: 22
                Layout.preferredHeight: 22
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
                        border.color: root.nameError ? "#e74c3c" : (nameField.activeFocus ? "#3584e4" : "#3d3d3d")
                        border.width: (root.nameError || nameField.activeFocus) ? 2 : 1
                        radius: 4
                    }

                    onTextChanged: {
                        if (root.nameError) {
                            root.nameError = false
                            root.errorText = ""
                        }
                    }

                    onAccepted: cmdField.forceActiveFocus()
                    Keys.onReturnPressed: function(event) {
                        if (event.modifiers & Qt.ControlModifier) {
                            root.submit()
                        } else {
                            cmdField.forceActiveFocus()
                        }
                    }
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
                        border.color: root.cmdError ? "#e74c3c" : (cmdField.activeFocus ? "#3584e4" : "#3d3d3d")
                        border.width: (root.cmdError || cmdField.activeFocus) ? 2 : 1
                        radius: 4
                    }

                    onTextChanged: {
                        if (root.cmdError) {
                            root.cmdError = false
                            root.errorText = ""
                        }
                    }

                    onAccepted: dirField.forceActiveFocus()
                    Keys.onReturnPressed: function(event) {
                        if (event.modifiers & Qt.ControlModifier) {
                            root.submit()
                        } else {
                            dirField.forceActiveFocus()
                        }
                    }
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
                    Keys.onReturnPressed: function(event) {
                        if (event.modifiers & Qt.ControlModifier) {
                            root.submit()
                        } else {
                            groupField.forceActiveFocus()
                        }
                    }
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
                    Keys.onReturnPressed: function(event) {
                        root.submit()
                    }
                }
            }
        }

        // Error message row
        Text {
            text: root.errorText
            color: "#ff6b6b"
            font.pixelSize: 11
            visible: root.errorText !== ""
            Layout.fillWidth: true
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
