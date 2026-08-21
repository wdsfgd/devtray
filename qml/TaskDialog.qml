import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Dialog {
    id: root
    property string taskId: ""
    property alias taskName: nameField.text
    property alias command: cmdField.text
    property alias workingDir: dirField.text
    property alias group: groupField.text

    signal saved(string id, string name, string command, string workingDir, string group)

    title: taskId === "" ? "Add Task" : "Edit Task"
    modal: true
    standardButtons: Dialog.Ok | Dialog.Cancel

    onAccepted: {
        root.saved(root.taskId, root.taskName, root.command, root.workingDir, root.group)
    }

    ColumnLayout {
        spacing: 8
        width: 350

        Label { text: "Task Name:" }
        TextField { id: nameField; Layout.fillWidth: true; placeholderText: "e.g. Frontend" }

        Label { text: "Command:" }
        TextField { id: cmdField; Layout.fillWidth: true; placeholderText: "e.g. npm run dev" }

        Label { text: "Working Directory:" }
        TextField { id: dirField; Layout.fillWidth: true; text: "." }

        Label { text: "Group (Optional):" }
        TextField { id: groupField; Layout.fillWidth: true; placeholderText: "e.g. Web" }
    }
}
