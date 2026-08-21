import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import devtray 1.0

ApplicationWindow {
    id: window
    width: 550
    height: 650
    visible: true
    title: "DevTray"
    color: "#1e1e1e"

    TaskManagerBridge {
        id: taskBridge
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 15
        spacing: 12

        RowLayout {
            Layout.fillWidth: true

            Label {
                text: "DevTray Tasks"
                font.pixelSize: 18
                font.bold: true
                color: "#ffffff"
                Layout.fillWidth: true
            }

            Button {
                text: "+ Add Task"
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

        ListView {
            id: taskListView
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8
            clip: true
            model: taskBridge ? taskBridge.tasks : []

            delegate: TaskCard {
                width: taskListView.width
                task: modelData
                isRunning: taskBridge ? taskBridge.isTaskRunning(modelData.id) : false

                onToggleClicked: {
                    if (isRunning) {
                        taskBridge.stopTask(modelData.id)
                    } else {
                        taskBridge.startTask(modelData.id)
                    }
                }
                onLogsClicked: {
                    logViewer.taskName = modelData.name
                    logViewer.logContent = taskBridge.getRecentLogs(modelData.name).join("\n")
                    logViewer.open()
                }
                onEditClicked: {
                    taskDialog.taskId = modelData.id
                    taskDialog.taskName = modelData.name
                    taskDialog.command = modelData.command
                    taskDialog.workingDir = modelData.working_directory
                    taskDialog.group = modelData.group || ""
                    taskDialog.open()
                }
                onMoveUpClicked: taskBridge.moveTask(modelData.id, -1)
                onMoveDownClicked: taskBridge.moveTask(modelData.id, 1)
                onDeleteClicked: taskBridge.deleteTask(modelData.id)
            }
        }
    }

    TaskDialog {
        id: taskDialog
        onSaved: function(id, name, command, dir, group) {
            taskBridge.saveTask(id, name, command, dir, group)
        }
    }

    LogViewer {
        id: logViewer
    }
}
