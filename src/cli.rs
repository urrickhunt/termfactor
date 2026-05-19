use crate::error::AppError;
use crate::leaderboard::TerminalFeature;

#[derive(Debug, Eq, PartialEq)]
pub enum CliCommand {
    Run,
    Bash,
    PrintBash,
    Version,
    Help,
    UpdateLeaderboard(UpdateLeaderboardArgs),
}

#[derive(Debug, Eq, PartialEq)]
pub struct UpdateLeaderboardArgs {
    pub os: String,
    pub terminal: String,
    pub score: u16,
    pub feature: TerminalFeature,
}

pub fn parse() -> Result<CliCommand, AppError> {
    parse_from(std::env::args().skip(1))
}

pub fn print_help() {
    println!("{}", usage_text());
}

const fn usage_text() -> &'static str {
    "usage: termfactor [OPTIONS]\noptions:\n  --bash       run embedded bash for terminals like mintty\n  --version    show version information\n  --help       show this help message"
}

fn parse_from<I>(args: I) -> Result<CliCommand, AppError>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();

    match args.as_slice() {
        [] => Ok(CliCommand::Run),
        [flag] if flag == "--bash" => Ok(CliCommand::Bash),
        [flag] if flag == "--print-bash" => Ok(CliCommand::PrintBash),
        [flag] if flag == "--version" => Ok(CliCommand::Version),
        [flag] if flag == "--help" => Ok(CliCommand::Help),
        [flag, os, terminal, score_text, feature_text] if flag == "--update-terminals" => {
            let score = score_text
                .parse::<u16>()
                .map_err(|_| AppError::message(format!("invalid points: {score_text}")))?;
            let feature = feature_text
                .parse::<TerminalFeature>()
                .map_err(|()| AppError::message(format!("invalid feature: {feature_text}")))?;

            Ok(CliCommand::UpdateLeaderboard(UpdateLeaderboardArgs {
                os: os.clone(),
                terminal: terminal.clone(),
                score,
                feature,
            }))
        }
        _ => Err(AppError::message(format!(
            "invalid arguments.\n\n{}",
            usage_text()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_from, CliCommand, UpdateLeaderboardArgs};
    use crate::leaderboard::TerminalFeature;

    #[test]
    fn parses_default_run() {
        assert_eq!(parse_from(Vec::new()).unwrap(), CliCommand::Run);
    }

    #[test]
    fn parses_update_command() {
        let args = vec![
            "--update-terminals".to_string(),
            "macOS 14.5".to_string(),
            "WezTerm".to_string(),
            "115".to_string(),
            "sick".to_string(),
        ];

        assert_eq!(
            parse_from(args).unwrap(),
            CliCommand::UpdateLeaderboard(UpdateLeaderboardArgs {
                os: "macOS 14.5".to_string(),
                terminal: "WezTerm".to_string(),
                score: 115,
                feature: TerminalFeature::Sick,
            })
        );
    }

    #[test]
    fn parses_print_bash() {
        assert_eq!(
            parse_from(vec!["--print-bash".to_string()]).unwrap(),
            CliCommand::PrintBash
        );
    }
}
