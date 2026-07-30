package main

import (
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"syscall"
)

type Task struct {
	Name             string `json:"name"`
	Command          string `json:"command"`
	WorkingDirectory string `json:"working_directory"`
	Group            string `json:"group,omitempty"`
}

type TaskManager struct {
	configPath string
	logDir     string
	Tasks      []*Task
	processes  map[*Task]*exec.Cmd
}

func NewTaskManager() *TaskManager {
	home, _ := os.UserHomeDir()
	tm := &TaskManager{
		configPath: filepath.Join(home, ".config", "devtray", "config.json"),
		logDir:     filepath.Join(home, ".cache", "devtray", "logs"),
		Tasks:      []*Task{},
		processes:  make(map[*Task]*exec.Cmd),
	}
	tm.LoadConfig()
	return tm
}

func (tm *TaskManager) LoadConfig() {
	data, err := os.ReadFile(tm.configPath)
	if err != nil {
		tm.Tasks = []*Task{}
		return
	}
	var tasks []*Task
	err = json.Unmarshal(data, &tasks)
	if err != nil {
		tm.Tasks = []*Task{}
		return
	}
	tm.Tasks = tasks
}

func (tm *TaskManager) SaveConfig() {
	os.MkdirAll(filepath.Dir(tm.configPath), 0755)
	data, _ := json.MarshalIndent(tm.Tasks, "", "  ")
	os.WriteFile(tm.configPath, data, 0644)
}

func (tm *TaskManager) AddTask(task *Task) {
	tm.Tasks = append(tm.Tasks, task)
	tm.SaveConfig()
}

func (tm *TaskManager) RemoveTask(task *Task) {
	for i, t := range tm.Tasks {
		if t == task {
			tm.StopTask(task)
			tm.Tasks = append(tm.Tasks[:i], tm.Tasks[i+1:]...)
			tm.SaveConfig()
			return
		}
	}
}

func (tm *TaskManager) UpdateTask(oldTask *Task, newTask *Task) {
	for i, t := range tm.Tasks {
		if t == oldTask {
			wasRunning := tm.IsRunning(oldTask)
			if wasRunning {
				tm.StopTask(oldTask)
			}
			tm.Tasks[i] = newTask
			tm.SaveConfig()
			if wasRunning {
				tm.StartTask(newTask)
			}
			return
		}
	}
}

func (tm *TaskManager) StartTask(task *Task) {
	if tm.IsRunning(task) {
		return
	}
	os.MkdirAll(tm.logDir, 0755)
	logFile := filepath.Join(tm.logDir, task.Name+".log")
	f, _ := os.OpenFile(logFile, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0644)

	cwd := task.WorkingDirectory
	if cwd == "" {
		cwd, _ = os.Getwd()
	} else {
		cwd = os.ExpandEnv(cwd)
		if len(cwd) > 0 && cwd[0] == '~' {
			home, _ := os.UserHomeDir()
			if len(cwd) == 1 {
				cwd = home
			} else if cwd[1] == '/' {
				cwd = filepath.Join(home, cwd[2:])
			}
		}
	}

	cmd := exec.Command("sh", "-c", task.Command)
	cmd.Dir = cwd
	cmd.Stdout = f
	cmd.Stderr = f
	// Create new process group
	cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}

	err := cmd.Start()
	if err == nil {
		tm.processes[task] = cmd
		// Wait in background to reap
		go func() {
			cmd.Wait()
			f.Close()
		}()
	} else {
		f.Close()
	}
}

func (tm *TaskManager) StopTask(task *Task) {
	if cmd, ok := tm.processes[task]; ok {
		if cmd.Process != nil {
			// Kill process group forcefully
			syscall.Kill(-cmd.Process.Pid, syscall.SIGKILL)
		}
		delete(tm.processes, task)
	}
}

func (tm *TaskManager) IsRunning(task *Task) bool {
	if cmd, ok := tm.processes[task]; ok {
		// ProcessState is set when Wait() completes
		if cmd.ProcessState != nil && cmd.ProcessState.Exited() {
			delete(tm.processes, task)
			return false
		}
		// If Process is set but ProcessState is nil, it's running
		if cmd.Process != nil && cmd.ProcessState == nil {
			return true
		}
	}
	return false
}

func (tm *TaskManager) StopAll() {
	for _, t := range tm.Tasks {
		tm.StopTask(t)
	}
}
