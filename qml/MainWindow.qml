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
        onTasksChanged: {
            window.rebuildTrayMenu()
        }
    }

    Component.onCompleted: {
        taskBridge.refreshTasks()
        window.rebuildTrayMenu()
    }

    onClosing: function(close) {
        close.accepted = false
        window.hide()
    }

    function refreshAll() {
        taskBridge.refreshTasks()
        window.rebuildTrayMenu()
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
            var count = taskBridge.runningCount()
            return count > 0 ? ("DevTray (" + count + " active)") : "DevTray"
        }

        onActivated: function(reason) {
            window.show()
            window.raise()
            window.requestActivate()
        }

        menu: Labs.Menu {
            id: trayMenu
        }
    }

    function rebuildTrayMenu() {
        trayMenu.clear()

        // Open Main Window Item
        var openItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', trayMenu)
        openItem.text = "Open Main Window"
        openItem.triggered.connect(function() {
            window.show()
            window.raise()
            window.requestActivate()
        })
        trayMenu.addItem(openItem)

        // Separator
        var sep1 = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuSeparator {}', trayMenu)
        trayMenu.addItem(sep1)

        var tasks = taskBridge ? taskBridge.tasks : []
        if (!tasks || tasks.length === 0) {
            var emptyItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', trayMenu)
            emptyItem.text = "No Tasks Available"
            emptyItem.enabled = false
            trayMenu.addItem(emptyItem)
        } else {
            var groups = {}
            var uncategorized = []
            var groupNames = []

            for (var i = 0; i < tasks.length; i++) {
                var t = tasks[i]
                var g = t.group ? t.group.trim() : ""
                if (g !== "") {
                    if (!groups[g]) {
                        groups[g] = []
                        groupNames.push(g)
                    }
                    groups[g].push(t)
                } else {
                    uncategorized.push(t)
                }
            }
            groupNames.sort()

            // Build Group Submenus
            for (var gi = 0; gi < groupNames.length; gi++) {
                var gName = groupNames[gi]
                var gTasks = groups[gName]

                var subMenu = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.Menu {}', trayMenu)
                subMenu.title = gName

                // ▶️ Start All
                var startAllItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', subMenu)
                startAllItem.text = "▶️ Start All"
                ;(function(group) {
                    startAllItem.triggered.connect(function() {
                        taskBridge.startGroup(group)
                        window.refreshAll()
                    })
                })(gName)
                subMenu.addItem(startAllItem)

                // 🛑 Stop All
                var stopAllItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', subMenu)
                stopAllItem.text = "🛑 Stop All"
                ;(function(group) {
                    stopAllItem.triggered.connect(function() {
                        taskBridge.stopGroup(group)
                        window.refreshAll()
                    })
                })(gName)
                subMenu.addItem(stopAllItem)

                var subSep = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuSeparator {}', subMenu)
                subMenu.addItem(subSep)

                // Task items in group
                for (var ti = 0; ti < gTasks.length; ti++) {
                    var task = gTasks[ti]
                    var taskItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', subMenu)
                    taskItem.text = task.name
                    taskItem.checkable = true
                    taskItem.checked = taskBridge.isTaskRunning(task.id)
                    ;(function(taskId) {
                        taskItem.triggered.connect(function() {
                            if (taskBridge.isTaskRunning(taskId)) {
                                taskBridge.stopTask(taskId)
                            } else {
                                taskBridge.startTask(taskId)
                            }
                            window.refreshAll()
                        })
                    })(task.id)
                    subMenu.addItem(taskItem)
                }

                trayMenu.addMenu(subMenu)
            }

            if (groupNames.length > 0 && uncategorized.length > 0) {
                var sepUncat = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuSeparator {}', trayMenu)
                trayMenu.addItem(sepUncat)
            }

            // Uncategorized tasks
            for (var ui = 0; ui < uncategorized.length; ui++) {
                var uTask = uncategorized[ui]
                var uItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', trayMenu)
                uItem.text = uTask.name
                uItem.checkable = true
                uItem.checked = taskBridge.isTaskRunning(uTask.id)
                ;(function(taskId) {
                    uItem.triggered.connect(function() {
                        if (taskBridge.isTaskRunning(taskId)) {
                            taskBridge.stopTask(taskId)
                        } else {
                            taskBridge.startTask(taskId)
                        }
                        window.refreshAll()
                    })
                })(uTask.id)
                trayMenu.addItem(uItem)
            }
        }

        // Separator
        var sep2 = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuSeparator {}', trayMenu)
        trayMenu.addItem(sep2)

        // Quit DevTray
        var quitItem = Qt.createQmlObject('import Qt.labs.platform 1.1 as Labs; Labs.MenuItem {}', trayMenu)
        quitItem.text = "Quit DevTray"
        quitItem.triggered.connect(function() {
            window.show()
            window.raise()
            window.requestActivate()
            quitConfirmDialog.open()
        })
        trayMenu.addItem(quitItem)
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
        ScrollView {
            id: scrollView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: taskListView
                anchors.fill: parent
                spacing: 4
                model: taskBridge ? taskBridge.tasks : []

                // Section Headers (Hermes & OmniRoute, Uncategorized)
                section.property: "group"
                section.criteria: ViewSection.FullString
                section.delegate: Component {
                    Item {
                        width: taskListView.width
                        height: 32

                        Text {
                            anchors.left: parent.left
                            anchors.leftMargin: 2
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 5
                            text: section !== "" ? section : "Uncategorized"
                            color: "#ffffff"
                            font.bold: true
                            font.pixelSize: 13
                        }
                    }
                }

                delegate: TaskCard {
                    width: taskListView.width
                    task: modelData
                    isRunning: modelData.is_running !== undefined ? modelData.is_running : (taskBridge ? taskBridge.isTaskRunning(modelData.id) : false)

                    onToggleClicked: {
                        if (isRunning) {
                            taskBridge.stopTask(modelData.id)
                        } else {
                            taskBridge.startTask(modelData.id)
                        }
                        window.refreshAll()
                    }

                    onLogsClicked: {
                        logViewer.taskName = modelData.name
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

                    onMoveUpClicked: {
                        taskBridge.moveTask(modelData.id, -1)
                        window.refreshAll()
                    }

                    onMoveDownClicked: {
                        taskBridge.moveTask(modelData.id, 1)
                        window.refreshAll()
                    }

                    onDeleteClicked: {
                        deleteConfirmDialog.contextData = modelData.id
                        deleteConfirmDialog.message = "Delete task '" + modelData.name + "'?"
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
    }

    // Add / Edit Task Dialog
    TaskDialog {
        id: taskDialog
        onSaved: function(id, name, command, dir, group) {
            taskBridge.saveTask(id, name, command, dir, group)
            window.refreshAll()
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
                window.refreshAll()
            }
        }
    }

    // Quit DevTray Confirmation Dialog
    ConfirmDialog {
        id: quitConfirmDialog
        dialogTitle: "Quit DevTray"
        message: "Quit DevTray?"
        subMessage: "This will terminate all running tasks. Are you sure you want to quit?"
        confirmButtonText: "Quit"
        isDestructive: true

        onConfirmed: {
            taskBridge.stopAll()
            Qt.quit()
        }
    }
}
