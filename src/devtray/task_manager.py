import os
import json
import subprocess
from pathlib import Path

class Task:
    def __init__(self, name: str, command: str, working_directory: str):
        self.name = name
        self.command = command
        self.working_directory = working_directory
        
    def to_dict(self):
        return {
            "name": self.name,
            "command": self.command,
            "working_directory": self.working_directory
        }
        
    @classmethod
    def from_dict(cls, data: dict):
        return cls(
            name=data.get("name", "Unknown"),
            command=data.get("command", ""),
            working_directory=data.get("working_directory", os.getcwd())
        )

class TaskManager:
    def __init__(self, config_path: str = None, log_dir: str = None):
        self.config_path = Path(config_path) if config_path else Path.home() / ".config" / "devtray" / "config.json"
        self.log_dir = Path(log_dir) if log_dir else Path.home() / ".cache" / "devtray" / "logs"
        self.tasks = []
        self._processes = {} # map Task object id to (subprocess.Popen, log_file_handle)
        
        self.load_config()

    def load_config(self):
        if not self.config_path.exists():
            self.tasks = []
            return
            
        try:
            with open(self.config_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                self.tasks = [Task.from_dict(t) for t in data]
        except (json.JSONDecodeError, FileNotFoundError):
            self.tasks = []
            
    def save_config(self):
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        data = [t.to_dict() for t in self.tasks]
        with open(self.config_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)

    def get_tasks(self):
        return self.tasks

    def add_task(self, task: Task):
        self.tasks.append(task)
        
    def remove_task(self, task: Task):
        if task in self.tasks:
            self.stop_task(task)
            self.tasks.remove(task)

    def start_task(self, task: Task):
        if self.is_running(task):
            return # already running
            
        self.log_dir.mkdir(parents=True, exist_ok=True)
        log_file_path = self.log_dir / f"{task.name}.log"
        
        # Open log file in append mode (or write mode)
        log_file = open(log_file_path, "w", encoding="utf-8")
        
        cwd = task.working_directory
        if not os.path.exists(cwd):
            cwd = os.getcwd()
            
        process = subprocess.Popen(
            task.command,
            shell=True,
            cwd=cwd,
            stdout=log_file,
            stderr=subprocess.STDOUT
        )
        
        self._processes[id(task)] = (process, log_file)
        
    def stop_task(self, task: Task):
        if id(task) in self._processes:
            process, log_file = self._processes[id(task)]
            process.terminate() # Send SIGTERM
            
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill() # Force kill if stubborn
                
            log_file.close()
            del self._processes[id(task)]
            
    def is_running(self, task: Task) -> bool:
        if id(task) in self._processes:
            process, _ = self._processes[id(task)]
            if process.poll() is None:
                return True
            else:
                # Process has finished, clean up state
                _, log_file = self._processes[id(task)]
                log_file.close()
                del self._processes[id(task)]
                return False
        return False
        
    def stop_all(self):
        for task in self.tasks:
            self.stop_task(task)
