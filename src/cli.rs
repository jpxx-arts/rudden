use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    Check,
    Add(AddArgs),
    Update(UpdateArgs),
    Rm(RmArgs),
    Show,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    #[arg(short, long)]
    pub message: String,
    #[arg(short, long)]
    pub importance: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    pub id: u32,
    #[arg(short, long)]
    pub status: Option<String>,
    #[arg(short, long)]
    pub importance: Option<String>,
}

#[derive(Args, Debug)]
pub struct RmArgs {
    pub id: u32,
}
