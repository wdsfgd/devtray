pub mod core;
pub mod gui;

slint::include_modules!();

#[cfg(test)]
mod tests {
    use super::*;
    use slint::{Model, SharedString, VecModel};
    use std::rc::Rc;

    #[test]
    fn test_slint_task_item_and_main_window_instantiation() {
        let window = MainWindow::new().expect("Failed to instantiate MainWindow");

        let task = TaskItem {
            id: SharedString::from("task-1"),
            name: SharedString::from("Web Server"),
            command: SharedString::from("npm run dev"),
            working_directory: SharedString::from("/path/to/web"),
            group: SharedString::from("Web"),
            is_running: true,
            can_move_up: false,
            can_move_down: true,
        };

        assert_eq!(task.name.as_str(), "Web Server");
        assert_eq!(task.command.as_str(), "npm run dev");
        assert!(task.is_running);

        let tasks_vec = vec![task];
        let model = Rc::new(VecModel::from(tasks_vec));
        window.set_tasks(model.clone().into());
        window.set_running_count(1);

        assert_eq!(window.get_running_count(), 1);
        assert_eq!(window.get_tasks().row_count(), 1);
    }
}
