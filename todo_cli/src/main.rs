use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{fs, io::Error};

#[derive(Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a new task
    Add {
        /// Title of the task to be created
        #[arg(value_name = "task-title")]
        title: String,
    },
    /// Lists all tasks
    List,
    /// Remove a task by its ID
    Remove {
        /// ID of the task to be removed
        #[arg(value_name = "task-id")]
        id: u32,
    },
    /// Mark a task as completed
    Complete {
        /// ID of the task to be marked completed
        #[arg(value_name = "task-id")]
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: u32,
    title: String,
    completed: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Self {
        return Task {
            id,
            title,
            completed: false,
        };
    }

    fn mark_completed(&mut self) {
        self.completed = true;
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskList {
    tasks: Vec<Task>,
    next_task_id: u32,
}

impl TaskList {
    fn new() -> Self {
        return TaskList {
            tasks: Vec::new(),
            next_task_id: 1,
        };
    }

    fn add_task(&mut self, title: String) {
        let task = Task::new(self.next_task_id, title);
        self.tasks.push(task);
        self.next_task_id += 1;
    }

    fn remove_task(&mut self, id: u32) {
        let index = self
            .tasks
            .iter()
            .enumerate()
            .find(|(_, task)| task.id == id)
            .map(|(index, _)| index);

        if let Some(index) = index {
            self.tasks.remove(index);
        }
    }

    fn get_task(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    fn list_tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
}

fn load_from_file(file_path: &str) -> TaskList {
    match fs::read_to_string(file_path) {
        Ok(file_content) => serde_json::from_str(&file_content).unwrap(),
        Err(..) => TaskList::new(),
    }
}

fn save_to_file(task_list: &TaskList, file_path: &str) -> Result<(), Error> {
    let serialized = serde_json::to_string(task_list)?;
    fs::write(file_path, &serialized)
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let file_path = "task_list.json";
    let mut task_list = load_from_file(file_path);

    match args.command {
        Commands::Add { title } => {
            task_list.add_task(title);
            save_to_file(&task_list, file_path)
        }
        Commands::List => {
            let tasks = task_list.list_tasks();
            for task in tasks {
                let completion_symbol = if task.completed { "✓" } else { " " };
                println!("[{}] {}: {}", completion_symbol, task.id, task.title);
            }
            Ok(())
        }
        Commands::Remove { id } => {
            task_list.remove_task(id);
            save_to_file(&task_list, file_path)
        }
        Commands::Complete { id } => {
            if let Some(task) = task_list.get_task(id) {
                task.mark_completed();
                save_to_file(&task_list, file_path)
            } else {
                Ok(())
            }
        }
    }
}
