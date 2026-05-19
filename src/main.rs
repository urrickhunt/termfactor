mod cli;
mod embedded_bash;
mod error;
mod leaderboard;
mod paths;
mod platform;
mod suite;
mod system;
mod ui;

use std::process::ExitCode;

use cli::CliCommand;
use error::AppError;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(()) | Err(AppError::Quit) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), AppError> {
    match cli::parse()? {
        CliCommand::Run => run_interactive(),
        CliCommand::Bash => embedded_bash::execute(),
        CliCommand::PrintBash => {
            print!("{}", embedded_bash::render());
            Ok(())
        }
        CliCommand::Version => {
            println!("termfactor {VERSION}");
            Ok(())
        }
        CliCommand::Help => {
            cli::print_help();
            Ok(())
        }
        CliCommand::UpdateLeaderboard(args) => {
            leaderboard::update_file(&args.os, &args.terminal, args.score, args.feature)
        }
    }
}

fn run_interactive() -> Result<(), AppError> {
    let system_info = system::load()?;
    leaderboard::validate_identity(&system_info.os, &system_info.terminal)?;
    let summary = ui::run(&system_info)?;
    leaderboard::update_file(
        &system_info.os,
        &system_info.terminal,
        summary.score,
        summary.feature,
    )
}
