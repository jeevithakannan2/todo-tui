use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Delete all tasks and create a new encyption key ( Use this if you forgot your key ).
    #[arg(short, long)]
    pub reset: bool,
    /// Generate a new encryption key.
    #[arg(short, long)]
    pub generate_key: bool,
}
