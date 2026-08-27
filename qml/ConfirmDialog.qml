import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Popup {
    id: root

    property string dialogTitle: "Confirm"
    property string message: ""
    property string subMessage: ""
    property string confirmButtonText: "OK"
    property bool isDestructive: false
    property var contextData: null

    signal confirmed(var data)
    signal cancelled()

    modal: true
    focus: true
    dim: true
    closePolicy: Popup.CloseOnEscape
    padding: 0

    width: Math.min(parent ? parent.width - 24 : 360, 360)
    implicitHeight: mainLayout.implicitHeight + 30
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    onOpened: {
        confirmBtn.forceActiveFocus()
    }

    background: Rectangle {
        color: "#242424"
        border.color: "#3d3d3d"
        border.width: 1
        radius: 8
    }

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        anchors.margins: 14
        spacing: 12
        focus: true

        Keys.onReturnPressed: {
            root.confirmed(root.contextData)
            root.close()
        }

        Keys.onEscapePressed: {
            root.cancelled()
            root.close()
        }

        // Header
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: root.dialogTitle
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
                color: closeBtnArea.containsMouse ? "#3a3a3a" : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "✕"
                    color: "#999999"
                    font.pixelSize: 11
                }

                MouseArea {
                    id: closeBtnArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.cancelled()
                        root.close()
                    }
                }
            }
        }

        // Body
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 6

            Text {
                text: root.message
                color: "#ffffff"
                font.pixelSize: 13
                font.bold: true
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                visible: root.message !== "" && root.message.toLowerCase().trim() !== root.dialogTitle.toLowerCase().trim()
            }

            Text {
                text: root.subMessage
                color: "#aaaaaa"
                font.pixelSize: 12
                wrapMode: Text.Wrap
                Layout.fillWidth: true
                visible: root.subMessage !== ""
            }
        }

        Item {
            Layout.preferredHeight: 4
        }

        // Buttons
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
                implicitWidth: 70

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

                onClicked: {
                    root.cancelled()
                    root.close()
                }
            }

            Button {
                id: confirmBtn
                text: root.confirmButtonText
                implicitHeight: 30
                implicitWidth: 80

                background: Rectangle {
                    color: {
                        if (root.isDestructive) {
                            return confirmBtn.down ? "#8c1d18" : (confirmBtn.hovered ? "#c02a24" : "#a82420")
                        } else {
                            return confirmBtn.down ? "#1c60b3" : (confirmBtn.hovered ? "#2b7de0" : "#3584e4")
                        }
                    }
                    border.color: root.isDestructive ? "#c02a24" : "#3584e4"
                    border.width: 1
                    radius: 4
                }

                contentItem: Text {
                    text: confirmBtn.text
                    color: "#ffffff"
                    font.bold: true
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: {
                    root.confirmed(root.contextData)
                    root.close()
                }
            }
        }
    }
}
