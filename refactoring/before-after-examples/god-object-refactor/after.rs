// src/main.rs

use clap::Parser;
use std::{fs, io};

// Declara os módulos que farão parte do binário
mod cli;
mod commands;
mod task;

// Traz os tipos necessários para o escopo
use cli::{Cli, Mode};
use task::ToDoList;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let rudden_dir = "./.rudden";
    fs::create_dir_all(rudden_dir)?;
    let rudden_file_path = rudden_dir.to_string() + "/.rudden";

    let mut to_do_list = ToDoList::load(&rudden_file_path)?;

    // O match agora apenas delega para a função correta
    match cli.mode {
        Mode::Check => {
            commands::check_tasks(&mut to_do_list, &rudden_file_path)?;
        }
        Mode::Add(args) => {
            commands::add_task(&mut to_do_list, args, &rudden_file_path)?;
        }
        Mode::Update(args) => {
            commands::update_task(&mut to_do_list, args, &rudden_file_path)?;
        }
        Mode::Rm(args) => {
            commands::remove_task(&mut to_do_list, args, &rudden_file_path)?;
        }
        Mode::Show => {
            commands::show_tasks(&to_do_list);
        }
    }

    Ok(())
}