package main

import (
	"html"
	"os"
	"path/filepath"
	"sort"

	"github.com/dawidd6/go-appindicator"
	"github.com/gotk3/gotk3/glib"
	"github.com/gotk3/gotk3/gtk"
)

type App struct {
	tm        *TaskManager
	indicator *appindicator.Indicator
	menu      *gtk.Menu
	mainWindow *gtk.Window
	listBox   *gtk.ListBox
}

func main() {
	glib.SetPrgname("devtray")
	glib.SetApplicationName("DevTray")
	gtk.Init(nil)

	tm := NewTaskManager()
	app := &App{tm: tm}

	cwd, _ := os.Getwd()
	iconPath := filepath.Join(cwd, "assets", "icon.jpg")

	app.indicator = appindicator.New("devtray", iconPath, appindicator.CategoryApplicationStatus)
	app.indicator.SetStatus(appindicator.StatusActive)

	app.buildMenu()
	
	// Automatically open the main window when the application starts
	app.openMainWindow()

	gtk.Main()
}

func (a *App) buildMenu() {
	menu, _ := gtk.MenuNew()

	openItem, _ := gtk.MenuItemNewWithLabel("Open Main Window")
	openItem.Connect("activate", func() {
		a.openMainWindow()
	})
	menu.Append(openItem)

	sep1, _ := gtk.SeparatorMenuItemNew()
	menu.Append(sep1)

	if len(a.tm.Tasks) == 0 {
		empty, _ := gtk.MenuItemNewWithLabel("No Tasks Available")
		empty.SetSensitive(false)
		menu.Append(empty)
	} else {
		groups := make(map[string][]*Task)
		var uncategorized []*Task

		for _, t := range a.tm.Tasks {
			if t.Group != "" {
				groups[t.Group] = append(groups[t.Group], t)
			} else {
				uncategorized = append(uncategorized, t)
			}
		}

		for groupName, groupTasks := range groups {
			groupItem, _ := gtk.MenuItemNewWithLabel(groupName)
			submenu, _ := gtk.MenuNew()
			groupItem.SetSubmenu(submenu)

			startAll, _ := gtk.MenuItemNewWithLabel("▶️ Start All")
			gTasksStart := groupTasks
			startAll.Connect("activate", func() {
				for _, t := range gTasksStart {
					a.tm.StartTask(t)
				}
				a.buildMenu()
				a.updateMainWindow()
			})
			submenu.Append(startAll)

			stopAll, _ := gtk.MenuItemNewWithLabel("🛑 Stop All")
			gTasksStop := groupTasks
			stopAll.Connect("activate", func() {
				for _, t := range gTasksStop {
					a.tm.StopTask(t)
				}
				a.buildMenu()
				a.updateMainWindow()
			})
			submenu.Append(stopAll)

			sep, _ := gtk.SeparatorMenuItemNew()
			submenu.Append(sep)

			for _, t := range groupTasks {
				task := t
				item, _ := gtk.CheckMenuItemNewWithLabel(task.Name)
				item.SetActive(a.tm.IsRunning(task))
				item.Connect("activate", func() {
					if a.tm.IsRunning(task) {
						a.tm.StopTask(task)
					} else {
						a.tm.StartTask(task)
					}
					a.buildMenu()
					a.updateMainWindow()
				})
				submenu.Append(item)
			}
			menu.Append(groupItem)
		}

		if len(uncategorized) > 0 && len(groups) > 0 {
			sep, _ := gtk.SeparatorMenuItemNew()
			menu.Append(sep)
		}

		for _, t := range uncategorized {
			task := t
			item, _ := gtk.CheckMenuItemNewWithLabel(task.Name)
			item.SetActive(a.tm.IsRunning(task))
			item.Connect("activate", func() {
				if a.tm.IsRunning(task) {
					a.tm.StopTask(task)
				} else {
					a.tm.StartTask(task)
				}
				a.buildMenu()
				a.updateMainWindow()
			})
			menu.Append(item)
		}
	}

	sep2, _ := gtk.SeparatorMenuItemNew()
	menu.Append(sep2)

	quitItem, _ := gtk.MenuItemNewWithLabel("Quit DevTray")
	quitItem.Connect("activate", func() {
		a.confirmQuit()
	})
	menu.Append(quitItem)

	menu.ShowAll()
	a.menu = menu
	a.indicator.SetMenu(menu)
}

func (a *App) openMainWindow() {
	if a.mainWindow != nil {
		a.updateMainWindow()
		a.mainWindow.Present()
		return
	}

	win, _ := gtk.WindowNew(gtk.WINDOW_TOPLEVEL)
	win.SetTitle("DevTray - Task Management")
	win.SetDefaultSize(400, 500)
	
	cwd, _ := os.Getwd()
	iconPath := filepath.Join(cwd, "assets", "icon.svg")
	win.SetIconFromFile(iconPath)
	
	win.Connect("delete-event", func() bool {
		a.mainWindow = nil
		return false // destroy
	})

	vbox, _ := gtk.BoxNew(gtk.ORIENTATION_VERTICAL, 10)
	vbox.SetBorderWidth(10)
	win.Add(vbox)

	hbox, _ := gtk.BoxNew(gtk.ORIENTATION_HORIZONTAL, 10)
	label, _ := gtk.LabelNew("")
	label.SetMarkup("<b>Task List</b>")
	label.SetXAlign(0)
	hbox.PackStart(label, true, true, 0)

	btnAdd, _ := gtk.ButtonNewWithLabel("Add Task")
	btnAdd.Connect("clicked", func() {
		a.openTaskDialog(win, "Add Task", nil)
	})
	hbox.PackStart(btnAdd, false, false, 0)
	vbox.PackStart(hbox, false, false, 0)

	scrolled, _ := gtk.ScrolledWindowNew(nil, nil)
	scrolled.SetPolicy(gtk.POLICY_NEVER, gtk.POLICY_AUTOMATIC)
	vbox.PackStart(scrolled, true, true, 0)

	listbox, _ := gtk.ListBoxNew()
	listbox.SetSelectionMode(gtk.SELECTION_NONE)
	
	listbox.SetHeaderFunc(func(row, before *gtk.ListBoxRow) {
		groupName, _ := row.GetName()
		var beforeGroup string
		if before != nil {
			beforeGroup, _ = before.GetName()
		}
		if before == nil || groupName != beforeGroup {
			lbl, _ := gtk.LabelNew("")
			displayGroup := groupName
			if displayGroup == "" {
				displayGroup = "Uncategorized"
			}
			lbl.SetMarkup("<b>" + html.EscapeString(displayGroup) + "</b>")
			lbl.SetXAlign(0)
			lbl.SetMarginTop(10)
			lbl.SetMarginBottom(5)
			row.SetHeader(lbl)
		} else {
			row.SetHeader(nil)
		}
	})
	
	scrolled.Add(listbox)

	a.mainWindow = win
	a.listBox = listbox

	a.updateMainWindow()
	win.ShowAll()
}

func (a *App) updateMainWindow() {
	if a.mainWindow == nil {
		return
	}
	
	// Remove all children
	a.listBox.GetChildren().Foreach(func(item interface{}) {
		a.listBox.Remove(item.(*gtk.Widget))
	})

	// Group and sort tasks deterministically
	groups := make(map[string][]*Task)
	var uncategorized []*Task

	for _, t := range a.tm.Tasks {
		if t.Group != "" {
			groups[t.Group] = append(groups[t.Group], t)
		} else {
			uncategorized = append(uncategorized, t)
		}
	}

	var groupNames []string
	for k := range groups {
		groupNames = append(groupNames, k)
	}
	sort.Strings(groupNames)
	
	var orderedTasks []*Task
	for _, g := range groupNames {
		orderedTasks = append(orderedTasks, groups[g]...)
	}
	orderedTasks = append(orderedTasks, uncategorized...)
	
	a.tm.Tasks = orderedTasks

	for _, t := range a.tm.Tasks {
		task := t
		row, _ := gtk.ListBoxRowNew()
		row.SetName(task.Group)
		box, _ := gtk.BoxNew(gtk.ORIENTATION_HORIZONTAL, 10)
		box.SetBorderWidth(5)

		var statusEmoji string
		if a.tm.IsRunning(task) {
			statusEmoji = "🟢"
		} else {
			statusEmoji = "🔴"
		}
		lblStatus, _ := gtk.LabelNew(statusEmoji)
		box.PackStart(lblStatus, false, false, 0)

		lbl, _ := gtk.LabelNew(task.Name)
		lbl.SetXAlign(0)
		box.PackStart(lbl, true, true, 0)

		btnToggle, _ := gtk.ButtonNew()
		var btnIcon string
		if a.tm.IsRunning(task) {
			btnIcon = "media-playback-stop"
		} else {
			btnIcon = "media-playback-start"
		}
		btnImg, _ := gtk.ImageNewFromIconName(btnIcon, gtk.ICON_SIZE_BUTTON)
		btnToggle.SetImage(btnImg)
		btnToggle.SetTooltipText("Start/Stop Task")
		btnToggle.Connect("clicked", func() {
			if a.tm.IsRunning(task) {
				a.tm.StopTask(task)
			} else {
				a.tm.StartTask(task)
			}
			a.buildMenu()
			a.updateMainWindow()
		})
		box.PackStart(btnToggle, false, false, 0)

		btnEdit, _ := gtk.ButtonNewWithLabel("Edit")
		btnEdit.Connect("clicked", func() {
			a.openTaskDialog(a.mainWindow, "Edit Task", task)
		})
		box.PackStart(btnEdit, false, false, 0)

		// Move Up Button
		btnUp, _ := gtk.ButtonNewFromIconName("go-up", gtk.ICON_SIZE_BUTTON)
		btnUp.SetTooltipText("Move Up")
		btnUp.Connect("clicked", func() {
			a.moveTask(task, -1)
		})
		box.PackStart(btnUp, false, false, 0)

		// Move Down Button
		btnDown, _ := gtk.ButtonNewFromIconName("go-down", gtk.ICON_SIZE_BUTTON)
		btnDown.SetTooltipText("Move Down")
		btnDown.Connect("clicked", func() {
			a.moveTask(task, 1)
		})
		box.PackStart(btnDown, false, false, 0)

		btnDel, _ := gtk.ButtonNewWithLabel("Delete")
		btnDel.Connect("clicked", func() {
			a.confirmDelete(task)
		})
		box.PackStart(btnDel, false, false, 0)

		row.Add(box)
		a.listBox.Add(row)
	}
	a.listBox.ShowAll()
}

func (a *App) moveTask(task *Task, direction int) {
	// Find index of task
	idx := -1
	for i, t := range a.tm.Tasks {
		if t == task {
			idx = i
			break
		}
	}
	if idx == -1 {
		return
	}

	// Find the next/prev task in the SAME group
	targetIdx := -1
	if direction < 0 {
		// Move up
		for i := idx - 1; i >= 0; i-- {
			if a.tm.Tasks[i].Group == task.Group {
				targetIdx = i
				break
			}
		}
	} else {
		// Move down
		for i := idx + 1; i < len(a.tm.Tasks); i++ {
			if a.tm.Tasks[i].Group == task.Group {
				targetIdx = i
				break
			}
		}
	}

	if targetIdx != -1 {
		// Swap
		a.tm.Tasks[idx], a.tm.Tasks[targetIdx] = a.tm.Tasks[targetIdx], a.tm.Tasks[idx]
		a.tm.SaveConfig()
		a.updateMainWindow()
		a.buildMenu()
	}
}

func (a *App) openTaskDialog(parent *gtk.Window, title string, existingTask *Task) {
	dialog, _ := gtk.DialogNew()
	dialog.SetTitle(title)
	dialog.SetTransientFor(parent)
	dialog.AddButton("Cancel", gtk.RESPONSE_CANCEL)
	dialog.AddButton("Save", gtk.RESPONSE_OK)
	dialog.SetDefaultSize(300, 200)
	dialog.SetBorderWidth(10)

	contentArea, _ := dialog.GetContentArea()
	contentArea.SetSpacing(10)

	addInput := func(labelTxt, defaultTxt string) *gtk.Entry {
		lbl, _ := gtk.LabelNew(labelTxt)
		lbl.SetXAlign(0)
		contentArea.PackStart(lbl, false, false, 0)
		entry, _ := gtk.EntryNew()
		entry.SetText(defaultTxt)
		contentArea.PackStart(entry, false, false, 0)
		return entry
	}

	nameStr, cmdStr, dirStr, groupStr := "", "", ".", ""
	if existingTask != nil {
		nameStr = existingTask.Name
		cmdStr = existingTask.Command
		dirStr = existingTask.WorkingDirectory
		groupStr = existingTask.Group
	}

	entryName := addInput("Task Name:", nameStr)
	entryCmd := addInput("Command:", cmdStr)
	entryDir := addInput("Working Directory:", dirStr)
	entryGroup := addInput("Group (Optional):", groupStr)

	dialog.ShowAll()
	response := dialog.Run()

	if response == gtk.RESPONSE_OK {
		n, _ := entryName.GetText()
		c, _ := entryCmd.GetText()
		d, _ := entryDir.GetText()
		g, _ := entryGroup.GetText()
		
		if n != "" && c != "" {
			newTask := &Task{Name: n, Command: c, WorkingDirectory: d, Group: g}
			if existingTask == nil {
				a.tm.AddTask(newTask)
			} else {
				a.tm.UpdateTask(existingTask, newTask)
			}
			a.updateMainWindow()
			a.buildMenu()
		}
	}
	dialog.Destroy()
}

func (a *App) confirmDelete(task *Task) {
	dialog := gtk.MessageDialogNew(a.mainWindow, 0, gtk.MESSAGE_WARNING, gtk.BUTTONS_YES_NO, "Delete task '%s'?", task.Name)
	dialog.FormatSecondaryText("This will stop the task if running and delete it.")
	resp := dialog.Run()
	if resp == gtk.RESPONSE_YES {
		a.tm.RemoveTask(task)
		a.updateMainWindow()
		a.buildMenu()
	}
	dialog.Destroy()
}

func (a *App) confirmQuit() {
	dialog := gtk.MessageDialogNew(nil, 0, gtk.MESSAGE_QUESTION, gtk.BUTTONS_YES_NO, "Quit DevTray?")
	dialog.FormatSecondaryText("This will terminate all running tasks. Are you sure you want to quit?")
	resp := dialog.Run()
	dialog.Destroy()
	
	if resp == gtk.RESPONSE_YES {
		a.tm.StopAll()
		gtk.MainQuit()
		os.Exit(0)
	}
}
