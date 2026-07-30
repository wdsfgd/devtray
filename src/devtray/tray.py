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
from devtray.task_manager import TaskManager

class DevTrayApp:
    def __init__(self, task_manager: TaskManager):
        self.task_manager = task_manager
        
        # Setup AppIndicator
        self.indicator = AppIndicator.Indicator.new(
            "devtray",
            "utilities-terminal", # Default system icon
            AppIndicator.IndicatorCategory.APPLICATION_STATUS
        )
        self.indicator.set_status(AppIndicator.IndicatorStatus.ACTIVE)
        
        self.build_menu()
        
    def build_menu(self):
        menu = Gtk.Menu()
        
        tasks = self.task_manager.get_tasks()
        
        if not tasks:
            item = Gtk.MenuItem(label="Belum ada Task")
            item.set_sensitive(False)
            menu.append(item)
        else:
            for task in tasks:
                is_running = self.task_manager.is_running(task)
                status_icon = "🟢" if is_running else "🔴"
                label_text = f"{status_icon} {task.name}"
                item = Gtk.MenuItem(label=label_text)
                # At this stage, it's read only. We'll add interactivity in Ticket 03
                item.set_sensitive(False)
                menu.append(item)
                
        menu.append(Gtk.SeparatorMenuItem())
        
        item_quit = Gtk.MenuItem(label="Quit DevTray")
        item_quit.connect("activate", self.quit)
        menu.append(item_quit)
        
        menu.show_all()
        self.indicator.set_menu(menu)
        
    def quit(self, widget):
        Gtk.main_quit()
        
    def run(self):
        Gtk.main()

if __name__ == "__main__":
    manager = TaskManager()
    app = DevTrayApp(manager)
    app.run()
