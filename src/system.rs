use std::env;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SystemInfo {
    pub os: String,
    pub shell: String,
    pub terminal: String,
    pub terminal_font: String,
}

pub fn load() -> Result<SystemInfo, AppError> {
    let output = fastfetch_output()?;
    let mut info = parse_fastfetch_output(&output);
    info.shell = detect_shell_info();
    Ok(info)
}

fn fastfetch_output() -> Result<String, AppError> {
    let output = Command::new("fastfetch")
        .args(["--config", "none", "--logo", "none"])
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::message(
                    "fastfetch is required but not installed. please install fastfetch & try again.",
                )
            } else {
                AppError::from(error)
            }
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(AppError::command_failed(
            "fastfetch",
            output.status,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

fn parse_fastfetch_output(output: &str) -> SystemInfo {
    let mut info = SystemInfo::default();

    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };

        let value = value.trim().to_string();

        if key.trim().eq_ignore_ascii_case("OS") {
            info.os = value;
        } else if key.trim().eq_ignore_ascii_case("Terminal") {
            info.terminal = value;
        } else if key.trim().eq_ignore_ascii_case("Terminal Font") {
            info.terminal_font = value;
        }
    }

    info
}

fn detect_shell_info() -> String {
    detect_configured_shell()
        .or_else(detect_powershell)
        .unwrap_or_else(|| "unknown shell".to_string())
}

fn detect_configured_shell() -> Option<String> {
    let shell_path = env::var("SHELL").ok().or_else(detect_comspec)?;

    let shell_name = Path::new(&shell_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(shell_path.as_str())
        .to_string();

    let version_output = shell_version_output(&shell_path, &shell_name)?;
    let version = extract_version(&version_output).unwrap_or_else(|| "unknown version".to_string());
    Some(format!("{shell_name} {version}"))
}

#[cfg(windows)]
fn detect_comspec() -> Option<String> {
    env::var("COMSPEC").ok()
}

#[cfg(not(windows))]
const fn detect_comspec() -> Option<String> {
    None
}

fn shell_version_output(shell_path: &str, shell_name: &str) -> Option<String> {
    let mut command = Command::new(shell_path);

    if shell_name.eq_ignore_ascii_case("cmd") || shell_name.eq_ignore_ascii_case("cmd.exe") {
        command.args(["/c", "ver"]);
    } else {
        command.arg("--version");
    }

    let output = command.output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Some(text)
}

fn detect_powershell() -> Option<String> {
    for program in ["pwsh", "powershell"] {
        let Ok(output) = Command::new(program)
            .args(["-Command", "$PSVersionTable.PSVersion.ToString()"])
            .output()
        else {
            continue;
        };

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !version.is_empty() {
            return Some(format!("{program} {version}"));
        }
    }

    None
}

fn extract_version(text: &str) -> Option<String> {
    let mut start = None;
    let mut saw_dot = false;
    let mut previous_was_dot = false;
    let mut valid_end = None;

    for (index, character) in text.char_indices() {
        if start.is_none() {
            if character.is_ascii_digit() {
                start = Some(index);
                saw_dot = false;
                previous_was_dot = false;
                valid_end = None;
            }
            continue;
        }

        if character.is_ascii_digit() {
            if saw_dot {
                valid_end = Some(index + character.len_utf8());
            }
            previous_was_dot = false;
        } else if character == '.' && !previous_was_dot {
            saw_dot = true;
            previous_was_dot = true;
        } else {
            if let (Some(begin), Some(end)) = (start, valid_end) {
                return Some(text[begin..end].to_string());
            }

            start = character.is_ascii_digit().then_some(index);
            saw_dot = false;
            previous_was_dot = false;
            valid_end = None;
        }
    }

    start
        .zip(valid_end)
        .map(|(begin, end)| text[begin..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_version, parse_fastfetch_output, SystemInfo};

    #[test]
    fn extracts_first_dotted_version() {
        assert_eq!(
            extract_version("bash, version 5.2.26(1)-release"),
            Some("5.2.26".to_string())
        );
    }

    #[test]
    fn extracts_versions_with_more_than_three_components() {
        assert_eq!(
            extract_version("Version 10.0.19045.5854"),
            Some("10.0.19045.5854".to_string())
        );
    }

    #[test]
    fn rejects_trailing_dot_versions() {
        assert_eq!(extract_version("bash version 5."), None);
    }

    #[test]
    fn parses_fastfetch_sections() {
        let output = "OS: Fedora Linux 40\nTerminal: WezTerm\nTerminal Font: JetBrains Mono";

        assert_eq!(
            parse_fastfetch_output(output),
            SystemInfo {
                os: "Fedora Linux 40".to_string(),
                shell: String::new(),
                terminal: "WezTerm".to_string(),
                terminal_font: "JetBrains Mono".to_string(),
            }
        );
    }
}
