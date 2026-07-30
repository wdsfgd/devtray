import gi

gi.require_version('Gtk', '3.0')
# Mute deprecation warnings by preferring AyatanaAppIndicator3
try:
    gi.require_version('AyatanaAppIndicator3', '0.1')
    from gi.repository import AyatanaAppIndicator3 as AppIndicator
except ValueError:
    gi.require_version('AppIndicator3', '0.1')
    from gi.repository import AppIndicator3 as AppIndicator

from gi.repository import Gtk, GLib
from devtray.task_manager import TaskManager, Task
from devtray.main_window import MainWindow

class DevTrayApp:
    def __init__(self, task_manager: TaskManager):
        self.task_manager = task_manager
        
        # Setup Main Window lazily to save memory
        self.main_window = None
        
        # Suppress stderr temporarily to hide the libayatana-appindicator C-level deprecation warning
        import os
        fd = os.open(os.devnull, os.O_WRONLY)
        old_fd = os.dup(2)
        os.dup2(fd, 2)
        
        try:
            self.indicator = AppIndicator.Indicator.new(
                "devtray",
                "utilities-terminal", # Default system icon
                AppIndicator.IndicatorCategory.APPLICATION_STATUS
            )
        finally:
            os.dup2(old_fd, 2)
            os.close(fd)
            os.close(old_fd)
            
        self.indicator.set_status(AppIndicator.IndicatorStatus.ACTIVE)
        
        # Build initial menu
        self.build_menu()
        
    def build_menu(self):
        menu = Gtk.Menu()
        
        item_open = Gtk.MenuItem(label="Buka Main Window")
        item_open.connect("activate", self.on_open_clicked)
        menu.append(item_open)
        
        menu.append(Gtk.SeparatorMenuItem())
        
        tasks = self.task_manager.get_tasks()
        
        if not tasks:
            item = Gtk.MenuItem(label="Belum ada Task")
            item.set_sensitive(False)
            menu.append(item)
        else:
            for task in tasks:
                is_running = self.task_manager.is_running(task)
                status_icon = "🛑 Stop" if is_running else "▶️ Start"
                label_text = f"{status_icon} {task.name}"
                
                item = Gtk.MenuItem(label=label_text)
                item.connect("activate", self.on_task_toggled, task)
                menu.append(item)
                
        menu.append(Gtk.SeparatorMenuItem())
        
        item_quit = Gtk.MenuItem(label="Quit DevTray")
        item_quit.connect("activate", self.on_quit_clicked)
        menu.append(item_quit)
        
        menu.show_all()
        self.indicator.set_menu(menu)
        
    def on_open_clicked(self, widget):
        if not self.main_window:
            self.main_window = MainWindow(self.task_manager, self.build_menu)
            self.main_window.connect("destroy", self.on_main_window_destroyed)
        self.main_window.update_ui()
        self.main_window.show_all()
        self.main_window.present()
        
    def on_main_window_destroyed(self, widget):
        self.main_window = None
        import gc
        gc.collect()
        
    def on_task_toggled(self, widget, task: Task):
        if self.task_manager.is_running(task):
            self.task_manager.stop_task(task)
        else:
            self.task_manager.start_task(task)
            
        # Rebuild the menu to reflect the new state immediately
        self.build_menu()
        
        # Also update main window if it's visible
        if self.main_window and self.main_window.get_visible():
            self.main_window.update_ui()
        
    def on_quit_clicked(self, widget):
        dialog = Gtk.MessageDialog(
            parent=None,
            flags=0,
            message_type=Gtk.MessageType.QUESTION,
            buttons=Gtk.ButtonsType.YES_NO,
            text="Keluar dari DevTray?"
        )
        dialog.format_secondary_text(
            "Ini akan mematikan semua Task yang sedang berjalan. Yakin ingin keluar?"
        )
        
        response = dialog.run()
        dialog.destroy()
        
        if response == Gtk.ResponseType.YES:
            # Gracefully terminate all background processes
            self.task_manager.stop_all()
            Gtk.main_quit()
            
    def run(self):
        Gtk.main()

if __name__ == "__main__":
    manager = TaskManager()
    app = DevTrayApp(manager)
    app.run()
