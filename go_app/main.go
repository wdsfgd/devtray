package main

import (
	"os"

	"github.com/dawidd6/go-appindicator"
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
	gtk.Init(nil)

	tm := NewTaskManager()
	app := &App{tm: tm}

	app.indicator = appindicator.New("devtray", "utilities-terminal", appindicator.CategoryApplicationStatus)
	app.indicator.SetStatus(appindicator.StatusActive)

	app.buildMenu()

	gtk.Main()
}

func (a *App) buildMenu() {
	menu, _ := gtk.MenuNew()

	openItem, _ := gtk.MenuItemNewWithLabel("Buka Main Window")
	openItem.Connect("activate", func() {
		a.openMainWindow()
	})
	menu.Append(openItem)

	sep1, _ := gtk.SeparatorMenuItemNew()
	menu.Append(sep1)

	if len(a.tm.Tasks) == 0 {
		empty, _ := gtk.MenuItemNewWithLabel("Belum ada Task")
		empty.SetSensitive(false)
		menu.Append(empty)
	} else {
		for _, t := range a.tm.Tasks {
			task := t // capture for closure
			label := "▶️ Start " + task.Name
			if a.tm.IsRunning(task) {
				label = "🛑 Stop " + task.Name
			}
			item, _ := gtk.MenuItemNewWithLabel(label)
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
	win.SetTitle("DevTray - Manajemen Task")
	win.SetDefaultSize(400, 500)
	win.Connect("delete-event", func() bool {
		a.mainWindow = nil
		return false // destroy
	})

	vbox, _ := gtk.BoxNew(gtk.ORIENTATION_VERTICAL, 10)
	vbox.SetBorderWidth(10)
	win.Add(vbox)

	hbox, _ := gtk.BoxNew(gtk.ORIENTATION_HORIZONTAL, 10)
	label, _ := gtk.LabelNew("")
	label.SetMarkup("<b>Daftar Task</b>")
	label.SetXAlign(0)
	hbox.PackStart(label, true, true, 0)

	btnAdd, _ := gtk.ButtonNewWithLabel("Tambah Task")
	btnAdd.Connect("clicked", func() {
		a.openTaskDialog(win, "Tambah Task", nil)
	})
	hbox.PackStart(btnAdd, false, false, 0)
	vbox.PackStart(hbox, false, false, 0)

	scrolled, _ := gtk.ScrolledWindowNew(nil, nil)
	scrolled.SetPolicy(gtk.POLICY_NEVER, gtk.POLICY_AUTOMATIC)
	vbox.PackStart(scrolled, true, true, 0)

	listbox, _ := gtk.ListBoxNew()
	listbox.SetSelectionMode(gtk.SELECTION_NONE)
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

	for _, t := range a.tm.Tasks {
		task := t
		row, _ := gtk.ListBoxRowNew()
		box, _ := gtk.BoxNew(gtk.ORIENTATION_HORIZONTAL, 10)
		box.SetBorderWidth(5)

		status := "🔴 "
		if a.tm.IsRunning(task) {
			status = "🟢 "
		}
		lbl, _ := gtk.LabelNew(status + task.Name)
		lbl.SetXAlign(0)
		box.PackStart(lbl, true, true, 0)

		btnEdit, _ := gtk.ButtonNewWithLabel("Edit")
		btnEdit.Connect("clicked", func() {
			a.openTaskDialog(a.mainWindow, "Edit Task", task)
		})
		box.PackStart(btnEdit, false, false, 0)

		btnDel, _ := gtk.ButtonNewWithLabel("Hapus")
		btnDel.Connect("clicked", func() {
			a.confirmDelete(task)
		})
		box.PackStart(btnDel, false, false, 0)

		row.Add(box)
		a.listBox.Add(row)
	}
	a.listBox.ShowAll()
}

func (a *App) openTaskDialog(parent *gtk.Window, title string, existingTask *Task) {
	dialog, _ := gtk.DialogNewWithButtons(title, parent, 0,
		[]interface{}{"Batal", gtk.RESPONSE_CANCEL, "Simpan", gtk.RESPONSE_OK})
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

	nameStr, cmdStr, dirStr := "", "", "."
	if existingTask != nil {
		nameStr = existingTask.Name
		cmdStr = existingTask.Command
		dirStr = existingTask.WorkingDirectory
	}

	entryName := addInput("Nama Task:", nameStr)
	entryCmd := addInput("Perintah (Command):", cmdStr)
	entryDir := addInput("Working Directory:", dirStr)

	dialog.ShowAll()
	response := dialog.Run()

	if response == gtk.RESPONSE_OK {
		n, _ := entryName.GetText()
		c, _ := entryCmd.GetText()
		d, _ := entryDir.GetText()
		
		if n != "" && c != "" {
			newTask := &Task{Name: n, Command: c, WorkingDirectory: d}
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
	dialog := gtk.MessageDialogNew(a.mainWindow, 0, gtk.MESSAGE_WARNING, gtk.BUTTONS_YES_NO, "Hapus task '%s'?", task.Name)
	dialog.FormatSecondaryText("Ini akan menghentikan task jika sedang berjalan dan menghapusnya.")
	resp := dialog.Run()
	if resp == gtk.RESPONSE_YES {
		a.tm.RemoveTask(task)
		a.updateMainWindow()
		a.buildMenu()
	}
	dialog.Destroy()
}

func (a *App) confirmQuit() {
	dialog := gtk.MessageDialogNew(nil, 0, gtk.MESSAGE_QUESTION, gtk.BUTTONS_YES_NO, "Keluar dari DevTray?")
	dialog.FormatSecondaryText("Ini akan mematikan semua Task yang sedang berjalan. Yakin ingin keluar?")
	resp := dialog.Run()
	dialog.Destroy()
	
	if resp == gtk.RESPONSE_YES {
		a.tm.StopAll()
		gtk.MainQuit()
		os.Exit(0)
	}
}
