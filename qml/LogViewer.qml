import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Popup {
    id: root

    property string taskName: ""
    property var taskBridge: null
    property int clearedLineCount: 0

    modal: true
    focus: true
    dim: true
    closePolicy: Popup.CloseOnEscape
    padding: 0

    width: Math.min(parent ? parent.width - 24 : 560, 560)
    height: Math.min(parent ? parent.height - 30 : 440, 440)
    x: parent ? Math.round((parent.width - width) / 2) : 0
    y: parent ? Math.round((parent.height - height) / 2) : 0

    background: Rectangle {
        color: "#242424"
        border.color: "#3d3d3d"
        border.width: 1
        radius: 8
    }

    onTaskNameChanged: {
        root.clearedLineCount = 0
        if (logTextArea) logTextArea.text = ""
        refreshLogs()
    }

    function formatLines(lines) {
        if (!lines || lines.length === 0) return ""
        var startIdx = 0
        if (root.clearedLineCount > 0) {
            if (lines.length > root.clearedLineCount) {
                startIdx = root.clearedLineCount
            } else {
                return ""
            }
        }
        var text = ""
        for (var i = startIdx; i < lines.length; i++) {
            var rawLine = String(lines[i] !== undefined ? lines[i] : "")
            // Strip terminal ANSI escape codes
            var cleanLine = rawLine.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, "")
            text += (text.length > 0 ? "\n" : "") + cleanLine
        }
        return text
    }

    function refreshLogs() {
        if (!root.visible || !root.taskBridge || root.taskName === "") return
        var lines = root.taskBridge.getRecentLogs(root.taskName)
        var newText = formatLines(lines)
        if (logTextArea.text !== newText) {
            var wasAtBottom = (logScrollView.ScrollBar.vertical.position + logScrollView.ScrollBar.vertical.size >= 0.92)
            var wasEmpty = logTextArea.text.length === 0
            logTextArea.text = newText
            if (wasAtBottom || wasEmpty || logScrollView.ScrollBar.vertical.size === 1.0) {
                logTextArea.cursorPosition = logTextArea.text.length
                logScrollView.ScrollBar.vertical.position = Math.max(0, 1.0 - logScrollView.ScrollBar.vertical.size)
            }
        }
    }

    onOpened: {
        root.clearedLineCount = 0
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
                text: "Live Logs: " + (root.taskName !== "" ? root.taskName : "Task")
                color: "#ffffff"
                font.bold: true
                font.pixelSize: 14
                Layout.fillWidth: true
                elide: Text.ElideRight
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
                    font.family: "DejaVu Sans Mono, Monospace, Courier, monospace"
                    font.pixelSize: 11
                    color: "#33d17a"
                    selectedTextColor: "#ffffff"
                    selectionColor: "#3584e4"
                    background: null
                    wrapMode: TextEdit.WrapAnywhere
                }
            }

            // Empty state placeholder
            Text {
                anchors.centerIn: parent
                text: "(No logs recorded yet...)"
                color: "#555555"
                font.pixelSize: 12
                font.italic: true
                visible: logTextArea.text.trim() === ""
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

                onClicked: {
                    if (root.taskBridge && root.taskName !== "") {
                        var currentLines = root.taskBridge.getRecentLogs(root.taskName)
                        root.clearedLineCount = currentLines ? currentLines.length : 0
                    }
                    logTextArea.text = ""
                }
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
