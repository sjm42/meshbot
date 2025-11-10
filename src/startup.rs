// startup.rs

use clap::Parser;

use crate::*;

#[derive(Clone, Debug, Default, Parser)]
pub struct OptsCommon {
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(short, long)]
    pub debug: bool,
    #[arg(short, long)]
    pub trace: bool,

    #[arg(long, short)]
    pub address: String
}

impl OptsCommon {
    pub fn get_loglevel(&self) -> Level {
        if self.trace {
            Level::TRACE
        } else if self.debug {
            Level::DEBUG
        } else if self.verbose {
            Level::INFO
        } else {
            Level::ERROR
        }
    }

    pub fn finish(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn start_pgm(&self, name: &str) {
        tracing_subscriber::fmt()
            .with_max_level(self.get_loglevel())
            .with_target(false)
            .init();
        info!("Starting up {name}...");
        // debug!("Git branch: {}", env!("GIT_BRANCH"));
        // debug!("Git commit: {}", env!("GIT_COMMIT"));
        // debug!("Source timestamp: {}", env!("SOURCE_TIMESTAMP"));
        debug!("Compiler version: {}", env!("RUSTC_VERSION"));
    }
}
// EOF
