import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.platform 1.1 as Labs
import devtray 1.0

ApplicationWindow {
    id: window
    width: 400
    height: 500
    minimumWidth: 360
    minimumHeight: 380
    visible: true
    title: "DevTray - Task Management"
    color: "#1e1e1e"

    TaskManagerBridge {
        id: taskBridge
    }

    Component.onCompleted: {
        taskBridge.refreshTasks()
    }

    onClosing: function(close) {
        if (systemTray && systemTray.available) {
            close.accepted = false
            window.hide()
        } else {
            taskBridge.stopAll()
            close.accepted = true
        }
    }

    function refreshAll() {
        taskBridge.refreshTasks()
    }

    // Periodic sync timer to update running states
    Timer {
        interval: 1000
        repeat: true
        running: true
        onTriggered: {
            taskBridge.refreshTasks()
        }
    }

    // System Tray Icon
    Labs.SystemTrayIcon {
        id: systemTray
        visible: true
        icon.source: taskBridge.iconPath()
        tooltip: {
            var count = taskBridge ? taskBridge.runningCount() : 0
            return count > 0 ? ("DevTray (" + count + " active)") : "DevTray"
        }

        onActivated: function(reason) {
            window.show()
            window.raise()
            window.requestActivate()
        }

        menu: Labs.Menu {
            id: trayMenu

            Labs.MenuItem {
                text: "Open Window"
                onTriggered: {
                    window.show()
                    window.raise()
                    window.requestActivate()
                }
            }

            Labs.MenuSeparator {}

            Labs.MenuItem {
                text: "Quit"
                onTriggered: {
                    window.show()
                    window.raise()
                    window.requestActivate()
                    quitConfirmDialog.open()
                }
            }
        }
    }

    // Main UI Layout
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        // Header Row ("Task List" + "Add Task" button)
        RowLayout {
            Layout.fillWidth: true

            Label {
                text: "Task List"
                font.pixelSize: 15
                font.bold: true
                color: "#ffffff"
                Layout.fillWidth: true
            }

            Button {
                id: btnAddTask
                text: "Add Task"
                implicitHeight: 28
                implicitWidth: 80

                background: Rectangle {
                    color: btnAddTask.down ? "#1f1f1f" : (btnAddTask.hovered ? "#3c3c3c" : "#2e2e2e")
                    border.color: btnAddTask.hovered ? "#555555" : "#444444"
                    radius: 4
                }

                contentItem: Text {
                    text: btnAddTask.text
                    color: "#ffffff"
                    font.bold: true
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                onClicked: {
                    taskDialog.taskId = ""
                    taskDialog.taskName = ""
                    taskDialog.command = ""
                    taskDialog.workingDir = "."
                    taskDialog.group = ""
                    taskDialog.open()
                }
            }
        }

        // Task List
        ListView {
            id: taskListView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 4
            model: taskBridge ? taskBridge.tasks : []
            boundsBehavior: Flickable.StopAtBounds

            ScrollBar.vertical: ScrollBar {
                id: vbar
                active: true
                policy: ScrollBar.AsNeeded
                width: 6
            }

            // Section Headers
            section.property: "group"
            section.criteria: ViewSection.FullString
            section.delegate: Component {
                Item {
                    width: taskListView.width
                    height: 28

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: 2
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 4
                        text: (section && section.trim() !== "") ? section : "Uncategorized"
                        color: "#999999"
                        font.bold: true
                        font.pixelSize: 12
                    }
                }
            }

            delegate: TaskCard {
                id: cardDelegate
                width: taskListView.width
                task: modelData
                listView: taskListView
                isRunning: (modelData && modelData.is_running !== undefined) ? modelData.is_running : (taskBridge && modelData ? taskBridge.isTaskRunning(modelData.id) : false)

                onToggleClicked: {
                    if (!cardDelegate.task) return
                    if (cardDelegate.isRunning) {
                        taskBridge.stopTask(cardDelegate.task.id)
                    } else {
                        taskBridge.startTask(cardDelegate.task.id)
                    }
                }

                onLogsClicked: {
                    if (!cardDelegate.task) return
                    logViewer.taskName = cardDelegate.task.name
                    logViewer.open()
                }

                onEditClicked: {
                    if (!cardDelegate.task) return
                    taskDialog.taskId = cardDelegate.task.id
                    taskDialog.taskName = cardDelegate.task.name
                    taskDialog.command = cardDelegate.task.command
                    taskDialog.workingDir = cardDelegate.task.working_directory
                    taskDialog.group = cardDelegate.task.group || ""
                    taskDialog.open()
                }

                onReorderRequested: function(taskId, targetIndex) {
                    taskBridge.reorderTask(taskId, targetIndex)
                }

                onDeleteClicked: {
                    if (!cardDelegate.task) return
                    deleteConfirmDialog.contextData = cardDelegate.task.id
                    deleteConfirmDialog.message = "Delete task '" + cardDelegate.task.name + "'?"
                    deleteConfirmDialog.subMessage = "This will stop the task if running and delete it."
                    deleteConfirmDialog.open()
                }
            }

            // Empty State
            Item {
                anchors.centerIn: parent
                width: parent.width
                height: 120
                visible: !taskBridge || !taskBridge.tasks || taskBridge.tasks.length === 0

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 8

                    Text {
                        text: "No tasks configured"
                        color: "#888888"
                        font.pixelSize: 14
                        font.bold: true
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "Click 'Add Task' to create your first task."
                        color: "#666666"
                        font.pixelSize: 12
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }
    }

    // Add / Edit Task Dialog
    TaskDialog {
        id: taskDialog
        onSaved: function(id, name, command, dir, group) {
            taskBridge.saveTask(id, name, command, dir, group)
        }
    }

    // Live Log Viewer Dialog
    LogViewer {
        id: logViewer
        taskBridge: taskBridge
    }

    // Delete Confirmation Dialog
    ConfirmDialog {
        id: deleteConfirmDialog
        dialogTitle: "Delete Task"
        confirmButtonText: "Delete"
        isDestructive: true

        onConfirmed: function(taskId) {
            if (taskId) {
                taskBridge.deleteTask(taskId)
            }
        }
    }

    // Quit DevTray Confirmation Dialog
    ConfirmDialog {
        id: quitConfirmDialog
        dialogTitle: "Quit DevTray"
        message: "Are you sure you want to quit?"
        subMessage: "This will terminate all running tasks."
        confirmButtonText: "Quit"
        isDestructive: true

        onConfirmed: {
            taskBridge.stopAll()
            Qt.quit()
        }
    }
}

