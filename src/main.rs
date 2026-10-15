use std::process::ExitCode;

use clap::Parser;
use cloneholio::{APP_NAME, cli::Args, config::Config, run};

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::resolve(Args::parse()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{APP_NAME}: error: {e}");
            return ExitCode::from(2);
        }
    };
    let progress = config.output.init_logging();
    match run::run(&config, progress).await {
        Ok(outcome) => outcome.exit_code(),
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        }
    }
}
