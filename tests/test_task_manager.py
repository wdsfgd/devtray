import os
import json
import time
import tempfile
from pathlib import Path
from devtray.task_manager import Task, TaskManager

def test_task_model():
    t = Task("Test Task", "echo 'hello'", "/tmp")
    assert t.name == "Test Task"
    assert t.command == "echo 'hello'"
    assert t.working_directory == "/tmp"
    
def test_task_manager_load_save(tmp_path):
    config_file = tmp_path / "config.json"
    log_dir = tmp_path / "logs"
    
    # Initialize with empty config
    manager = TaskManager(config_path=str(config_file), log_dir=str(log_dir))
    
    # Add tasks
    manager.add_task(Task("Task 1", "echo 1", "/tmp"))
    manager.add_task(Task("Task 2", "echo 2", "/tmp"))
    manager.save_config()
    
    # Reload
    manager2 = TaskManager(config_path=str(config_file), log_dir=str(log_dir))
    manager2.load_config()
    
    tasks = manager2.get_tasks()
    assert len(tasks) == 2
    assert tasks[0].name == "Task 1"
    assert tasks[1].command == "echo 2"

def test_task_manager_start_stop(tmp_path):
    config_file = tmp_path / "config.json"
    log_dir = tmp_path / "logs"
    
    manager = TaskManager(config_path=str(config_file), log_dir=str(log_dir))
    
    # A task that runs for 5 seconds
    task = Task("Sleep Task", "sleep 5", str(tmp_path))
    manager.add_task(task)
    
    assert not manager.is_running(task)
    
    # Start it
    manager.start_task(task)
    assert manager.is_running(task)
    
    # Stop it
    manager.stop_task(task)
    time.sleep(0.1) # give it a moment to die
    assert not manager.is_running(task)

def test_task_manager_logging(tmp_path):
    config_file = tmp_path / "config.json"
    log_dir = tmp_path / "logs"
    
    manager = TaskManager(config_path=str(config_file), log_dir=str(log_dir))
    
    # A task that outputs to stdout
    task = Task("Echo Task", "echo 'hello devtray'", str(tmp_path))
    manager.add_task(task)
    
    manager.start_task(task)
    time.sleep(0.2) # wait for execution to finish
    
    assert not manager.is_running(task)
    
    log_file = log_dir / "Echo Task.log"
    assert log_file.exists()
    
    content = log_file.read_text().strip()
    assert content == "hello devtray"
