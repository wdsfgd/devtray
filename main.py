import sys
from devtray.task_manager import TaskManager
from devtray.tray import DevTrayApp

def main():
    manager = TaskManager()
    app = DevTrayApp(manager)
    print("Memulai DevTray...")
    app.run()

if __name__ == "__main__":
    main()
