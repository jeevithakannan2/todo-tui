use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Delete all tasks and OS password start new.
    #[arg(short, long)]
    pub reset: bool,
}
