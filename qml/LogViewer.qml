import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Popup {
    id: root

    property string taskName: ""
    property var taskBridge: null

    modal: true
    focus: true
    dim: true
    closePolicy: Popup.CloseOnEscape

    width: 600
    height: 440
    x: Math.round((parent.width - width) / 2)
    y: Math.round((parent.height - height) / 2)

    background: Rectangle {
        color: "#242424"
        border.color: "#3d3d3d"
        border.width: 1
        radius: 8
    }

    function refreshLogs() {
        if (!root.visible || !root.taskBridge || root.taskName === "") return
        var lines = root.taskBridge.getRecentLogs(root.taskName)
        var newText = lines.join("\n")
        if (logTextArea.text !== newText) {
            var wasAtBottom = (logScrollView.ScrollBar.vertical.position + logScrollView.ScrollBar.vertical.size >= 0.95)
            logTextArea.text = newText
            if (wasAtBottom || logScrollView.ScrollBar.vertical.size === 1.0) {
                logTextArea.cursorPosition = logTextArea.text.length
            }
        }
    }

    onOpened: {
        refreshLogs()
        logTimer.start()
    }

    onClosed: {
        logTimer.stop()
    }

    Timer {
        id: logTimer
        interval: 500
        repeat: true
        running: false
        onTriggered: root.refreshLogs()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 14
        spacing: 10

        // Header
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: "Live Logs: " + root.taskName
                color: "#ffffff"
                font.bold: true
                font.pixelSize: 14
                Layout.fillWidth: true
                elide: Text.ElideRight
            }

            Rectangle {
                width: 22
                height: 22
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
                    onClicked: root.close()
                }
            }
        }

        // Terminal Log Box
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#141414"
            border.color: "#333333"
            border.width: 1
            radius: 4

            ScrollView {
                id: logScrollView
                anchors.fill: parent
                anchors.margins: 8
                clip: true

                TextArea {
                    id: logTextArea
                    readOnly: true
                    selectByMouse: true
                    font.family: "DejaVu Sans Mono, Monospace, Courier"
                    font.pixelSize: 11
                    color: "#33d17a"
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    background: null
                    wrapMode: TextEdit.WrapAnywhere
                }
            }
        }

        // Action Toolbar
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Button {
                id: clearBtn
                text: "Clear View"
                implicitHeight: 28
                implicitWidth: 85

                background: Rectangle {
                    color: clearBtn.down ? "#1f1f1f" : (clearBtn.hovered ? "#383838" : "#2c2c2c")
                    border.color: "#444444"
                    radius: 4
                }

                contentItem: Text {
                    text: clearBtn.text
                    color: "#dcdcdc"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: logTextArea.text = ""
            }

            Item {
                Layout.fillWidth: true
            }

            Button {
                id: closeBtn
                text: "Close"
                implicitHeight: 28
                implicitWidth: 70

                background: Rectangle {
                    color: closeBtn.down ? "#1f1f1f" : (closeBtn.hovered ? "#383838" : "#2c2c2c")
                    border.color: "#444444"
                    radius: 4
                }

                contentItem: Text {
                    text: closeBtn.text
                    color: "#dcdcdc"
                    font.pixelSize: 11
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: root.close()
            }
        }
    }
}
