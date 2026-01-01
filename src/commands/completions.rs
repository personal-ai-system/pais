use clap::CommandFactory;
use clap_complete::generate;
use eyre::Result;
use std::io;

use crate::cli::Cli;

pub fn run(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "paii", &mut io::stdout());
    Ok(())
}
