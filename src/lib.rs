pub mod cli;
pub mod commands;
pub mod task;

use std::fs;

use clap::Parser;

use crate::cli::{Cli, Mode};
use crate::task::ToDoList;

/// The main entry point for the Rudden application logic.
pub fn run() {
    // We use a helper function to easily bubble up errors with `?`.
    if let Err(e) = try_run() {
        // Use eprintln to write to standard error.
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}

/// Helper function to handle the main logic and propagate errors.
fn try_run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let rudden_dir = ".rudden";
    fs::create_dir_all(rudden_dir)?;
    let rudden_file_path = format!("{}/.rudden", rudden_dir);

    let mut to_do_list = ToDoList::load(&rudden_file_path)?;

    // The logic of each command now returns a Result<String, String>
    // which we can handle here.
    let command_result = match cli.mode {
        Mode::Add(ref args) => commands::add_task(&mut to_do_list, args),
        Mode::Update(ref args) => commands::update_task(&mut to_do_list, args),
        Mode::Rm(ref args) => commands::remove_task(&mut to_do_list, args),
        Mode::Show => commands::show_tasks(&to_do_list),
        // `check_tasks` returns an io::Result, so we map the error to a String.
        Mode::Check => commands::check_tasks(&mut to_do_list, &std::env::current_dir()?).map_err(|e| e.to_string()),
    };

    match command_result {
        Ok(success_message) => {
            println!("{}", success_message);
            // Determine if the state needs to be saved.
            // We don't save on `show` or if `check` didn't find anything.
            let should_save = !matches!(cli.mode, Mode::Show) &&
                              !(matches!(cli.mode, Mode::Check) && success_message.starts_with("No tasks"));
            if should_save {
                to_do_list.save(&rudden_file_path)?;
            }
        }
        Err(error_message) => {
            // Use eprintln for errors so they can be redirected.
            eprintln!("Error: {}", error_message);
            // Return a generic error to exit with a non-zero status code.
            return Err(Box::from(error_message));
        }
    }

    Ok(())
}