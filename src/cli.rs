use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Delete all tasks and OS password start new.
    #[arg(short, long)]
    pub reset: bool,
    /// Change the password used to encrypt tasks.
    #[arg(short, long)]
    pub change_password: bool,
}
