# Todo CLI in Rust

This tutorial will guide you through building a simple command-line interface (CLI) application in Rust, allowing users to add, list, remove, and mark tasks as completed.

## What you will learn

- Building a CLI application using [clap](https://docs.rs/clap/latest/clap/)
- Reading and writing data to the file system
- Serializing data using [serde](https://serde.rs/)

## Overview

This project demonstrates how to build a todo application over the command-line interface (CLI) using Rust. The application will allow users to add, list, remove, and mark tasks as completed.

## Features

- Add a new task to the list.
- List all tasks in the list.
- Remove a task from the list.
- Mark a task as completed.

## Walkthrough

1. Adding a task:
    - Implement a subcommand `add` that accepts a task title as a positional argument.
    - Add the task to the list using the specified task title.
    - Save the updated list to the file.
2. Listing tasks:
    - Implement a subcommand `list` that lists all tasks in the list.
    - Return the list of tasks to the standard output.
3. Removing a task:
    - Implement a subcommand `remove` that accepts a task ID as a positional argument.
    - Remove the task with the specified ID from the list.
    - Save the updated list to the file.
4. Marking a task as completed:
    - Implement a subcommand `complete` that accepts a task ID as a positional argument.
    - Mark the task with the specified ID as completed.
    - Save the updated list to the file.
