use clap::Parser;
use std::io::Result;

#[derive(Parser)]
pub struct Args {
    /// Delete all tasks and create a new encyption key ( Use this if you forgot your key ).
    #[arg(short, long)]
    pub reset: bool,
    /// Generate a new encryption key.
    #[arg(short, long)]
    pub generate_key: bool,
}

pub fn handle_arguments() -> Result<()> {
    let args = Args::parse();
    if args.reset {
        crate::tasks::reset()?;
        crate::auth::generate_key();
    }
    if args.generate_key {
        crate::auth::generate_key();
    }
    Ok(())
}
