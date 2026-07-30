import gi

gi.require_version('Gtk', '3.0')
from gi.repository import Gtk
from devtray.task_manager import TaskManager, Task

class AddTaskDialog(Gtk.Dialog):
    def __init__(self, parent):
        super().__init__(
            title="Tambah Task",
            parent=parent,
            flags=0,
            buttons=(
                Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL,
                Gtk.STOCK_ADD, Gtk.ResponseType.OK
            )
        )
        self.set_default_size(300, 200)
        self.set_border_width(10)
        
        box = self.get_content_area()
        box.set_spacing(10)
        
        # Name Input
        box.pack_start(Gtk.Label(label="Nama Task:", xalign=0), False, False, 0)
        self.entry_name = Gtk.Entry()
        box.pack_start(self.entry_name, False, False, 0)
        
        # Command Input
        box.pack_start(Gtk.Label(label="Perintah (Command):", xalign=0), False, False, 0)
        self.entry_cmd = Gtk.Entry()
        box.pack_start(self.entry_cmd, False, False, 0)
        
        # Directory Input
        box.pack_start(Gtk.Label(label="Working Directory:", xalign=0), False, False, 0)
        self.entry_dir = Gtk.Entry(text=".")
        box.pack_start(self.entry_dir, False, False, 0)
        
        self.show_all()

    def get_task(self):
        return Task(
            name=self.entry_name.get_text().strip(),
            command=self.entry_cmd.get_text().strip(),
            working_directory=self.entry_dir.get_text().strip()
        )

class MainWindow(Gtk.Window):
    def __init__(self, task_manager: TaskManager, on_tasks_changed_cb):
        super().__init__(title="DevTray - Manajemen Task")
        self.set_default_size(400, 500)
        self.task_manager = task_manager
        self.on_tasks_changed_cb = on_tasks_changed_cb
        
        # Sembunyikan window saat tombol close (X) ditekan
        self.connect("delete-event", self.on_delete_event)
        
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        vbox.set_border_width(10)
        self.add(vbox)
        
        # Header Box
        hbox = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        label = Gtk.Label(label="<b>Daftar Task</b>", use_markup=True, xalign=0)
        hbox.pack_start(label, True, True, 0)
        
        btn_add = Gtk.Button(label="Tambah Task")
        btn_add.connect("clicked", self.on_add_clicked)
        hbox.pack_start(btn_add, False, False, 0)
        
        vbox.pack_start(hbox, False, False, 0)
        
        # Scrollable List
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        vbox.pack_start(scrolled, True, True, 0)
        
        self.listbox = Gtk.ListBox()
        self.listbox.set_selection_mode(Gtk.SelectionMode.NONE)
        scrolled.add(self.listbox)
        
        self.update_ui()
        
    def on_delete_event(self, widget, event):
        self.hide()
        return True # Prevent destruction
        
    def update_ui(self):
        # Clear existing items
        for child in self.listbox.get_children():
            self.listbox.remove(child)
            
        tasks = self.task_manager.get_tasks()
        
        for task in tasks:
            row = Gtk.ListBoxRow()
            
            box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
            box.set_border_width(5)
            
            # Label
            is_running = self.task_manager.is_running(task)
            status = "🟢" if is_running else "🔴"
            lbl = Gtk.Label(label=f"{status} {task.name}", xalign=0)
            box.pack_start(lbl, True, True, 0)
            
            # Delete Button
            btn_del = Gtk.Button(label="Hapus")
            btn_del.connect("clicked", self.on_delete_clicked, task)
            box.pack_start(btn_del, False, False, 0)
            
            row.add(box)
            self.listbox.add(row)
            
        self.listbox.show_all()
        
    def on_add_clicked(self, widget):
        dialog = AddTaskDialog(self)
        response = dialog.run()
        
        if response == Gtk.ResponseType.OK:
            new_task = dialog.get_task()
            if new_task.name and new_task.command:
                self.task_manager.add_task(new_task)
                self.task_manager.save_config()
                self.update_ui()
                self.on_tasks_changed_cb() # Refresh tray menu
                
        dialog.destroy()
        
    def on_delete_clicked(self, widget, task: Task):
        dialog = Gtk.MessageDialog(
            parent=self,
            flags=0,
            message_type=Gtk.MessageType.WARNING,
            buttons=Gtk.ButtonsType.YES_NO,
            text=f"Hapus task '{task.name}'?"
        )
        dialog.format_secondary_text("Ini akan menghentikan task jika sedang berjalan dan menghapusnya dari konfigurasi.")
        
        response = dialog.run()
        if response == Gtk.ResponseType.YES:
            self.task_manager.remove_task(task)
            self.task_manager.save_config()
            self.update_ui()
            self.on_tasks_changed_cb() # Refresh tray menu
            
        dialog.destroy()
