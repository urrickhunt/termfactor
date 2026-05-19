use std::env;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;
use crate::leaderboard::TerminalFeature;
use crate::suite::{Check, Display, CHECKS, MAX_SCORE, POINTS_PER_CHECK};

const SCRIPT_PREFIX: &str = r#"#!/usr/bin/env bash
set -euo pipefail

RUST_BINARY_PATH="$1"
shift

trap 'printf "\033[0;33;3msession terminated by user\033[0m\n"; exit 0' SIGINT SIGTERM
"#;

const SCRIPT_SYSTEM_SETUP: &str = r#"
if ! command -v fastfetch >/dev/null 2>&1; then
    printf "fastfetch is required but not installed. please install fastfetch & try again.\n"
    exit 1
fi

ff_output=$(fastfetch --config none --logo none)
os_info=$(printf "%s\n" "$ff_output" | awk 'tolower($0) ~ /^os:[[:space:]]*/ { sub(/^[^:]*:[[:space:]]*/, "", $0); print; exit }')
terminal_font_info=$(printf "%s\n" "$ff_output" | awk 'tolower($0) ~ /^terminal font:[[:space:]]*/ { sub(/^[^:]*:[[:space:]]*/, "", $0); print; exit }')
terminal_info=$(printf "%s\n" "$ff_output" | awk 'tolower($0) ~ /^terminal:[[:space:]]*/ { sub(/^[^:]*:[[:space:]]*/, "", $0); print; exit }')

if [[ -z $os_info || -z $terminal_info ]]; then
    printf "terminal or OS information is unavailable, so the test cannot continue.\n"
    exit 1
fi

if [[ -n ${SHELL:-} && -x ${SHELL:-} ]]; then
    shell_name=$(basename "$SHELL")
    shell_version=$("$SHELL" --version 2>&1 | grep -oE '[0-9]+(\.[0-9]+)+' | head -n 1)
    if [[ -n $shell_version ]]; then
        shell_info="$shell_name $shell_version"
    else
        shell_info="$shell_name unknown version"
    fi
else
    shell_info="unknown shell"
fi

print_header() {
    printf "\e[1;36m# %s\e[0m\n" "$1"
}

reset_format() {
    printf '\e[0m'
}

confirm_prompt() {
    local prompt_message="$1"
    local escape_read_timeout=1
    local escape_drain_timeout=1
    local reset=$'\033[0m'
    if (( BASH_VERSINFO[0] >= 4 )); then
        escape_read_timeout=0.05
        escape_drain_timeout=0.01
    fi
    printf "%s (y/n or q): " "$prompt_message"
    while true; do
        if ! read -r -n 1 -s confirm; then
            printf "\033[0;33;3msession terminated by user%s\n" "$reset"
            exit 0
        fi

        if [[ $confirm == [yY] ]]; then
            printf "\033[0;32myes%s\n" "$reset"
            return 0
        fi

        if [[ $confirm == [nN] ]]; then
            printf "\033[0;31mno%s\n" "$reset"
            return 1
        fi

        if [[ $confirm == [qQ] ]]; then
            printf "\033[0;33;3msession terminated by user%s\n" "$reset"
            exit 0
        fi

        if [[ $confirm == $'\033' ]]; then
            if read -r -t "$escape_read_timeout" -n 1 -s; then
                while read -r -t "$escape_drain_timeout" -n 1 -s; do :; done
                continue
            fi

            printf "\033[0;33;3msession terminated by user%s\n" "$reset"
            exit 0
        fi
    done
}

print_header "terminal"
printf "OS: \033[34m%s\033[0m\n" "$os_info"
printf "shell: \e[32m%s\e[0m\n" "$shell_info"
printf "terminal font: \033[33m%s\033[0m\n" "$terminal_font_info"
printf "terminal: \e[1;35m%s\e[0m\n\n" "$terminal_info"

points=0

"#;

const GRADIENT_SCRIPT: &str = r#"for ((colnum = 0; colnum < 77; colnum++)); do
    r=$((255 - (colnum * 255 / 76)))
    g=$((colnum * 510 / 76))
    b=$((colnum * 255 / 76))
    if (( g > 255 )); then
        g=$((510 - g))
    fi

    if (( colnum % 2 == 0 )); then
        symbol="/"
    else
        symbol="\\"
    fi

    printf '\033[48;2;%s;%s;%sm\033[38;2;%s;%s;%sm%s' \
        "$r" "$g" "$b" "$((255 - r))" "$((255 - g))" "$((255 - b))" "$symbol"
done
printf '\033[0m\n\n'
"#;

pub fn execute() -> Result<(), AppError> {
    let script = render();
    let script_file = TempScript::new(&script)?;
    let current_executable = env::current_exe()?;

    let status = Command::new("bash")
        .arg(script_file.path())
        .arg(current_executable)
        .status()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::message("bash is required but not found in PATH.")
            } else {
                AppError::from(error)
            }
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::message(format!(
            "bash script exited with status {status}"
        )))
    }
}

pub fn render() -> String {
    let mut script = String::from(SCRIPT_PREFIX);
    let mut previous_header = None;

    writeln!(script, "MAX_SCORE={MAX_SCORE}").unwrap();
    writeln!(script, "POINTS_PER_CHECK={POINTS_PER_CHECK}").unwrap();
    script.push_str(SCRIPT_SYSTEM_SETUP);

    for check in CHECKS {
        append_check(&mut script, check, &mut previous_header);
    }

    script.push_str(
        r#"print_header "score"
printf "OS: \033[34m%s\033[0m\n" "$os_info"
printf "terminal: \e[1;35m%s\e[0m\n" "$terminal_info"
percentage=$((points * 100 / MAX_SCORE))
printf "total points: \e[32m%s\e[0m/%s (%s%%)\n" "$points" "$MAX_SCORE" "$percentage"
"#,
    );
    append_terminal_feature_summary(&mut script);
    script.push_str("printf \"\\n\\n\"\n");
    script.push_str("\"$RUST_BINARY_PATH\" --update-terminals \"$os_info\" \"$terminal_info\" \"$points\" \"$terminal_feature\"\n");

    script
}

fn append_terminal_feature_summary(script: &mut String) {
    let mut is_first_branch = true;

    for feature in TerminalFeature::ordered() {
        if let Some(condition) = feature.bash_condition() {
            let keyword = if is_first_branch { "if" } else { "elif" };
            writeln!(script, "{keyword} (( {condition} )); then").unwrap();
            is_first_branch = false;
        } else {
            writeln!(script, "else").unwrap();
        }

        let banner = format!("\u{1b}[1;36m{}\u{1b}[0m\n", feature.banner());
        writeln!(script, "    printf '%s' {}", bash_quote(&banner)).unwrap();
        writeln!(
            script,
            "    terminal_feature={}",
            bash_quote(feature.as_str())
        )
        .unwrap();
    }

    writeln!(script, "fi").unwrap();
}

fn append_check(script: &mut String, check: Check, previous_header: &mut Option<&'static str>) {
    if *previous_header != Some(check.header) {
        writeln!(script, "print_header {}", bash_quote(check.header)).unwrap();
        *previous_header = Some(check.header);
    }

    append_display(script, check.display);
    writeln!(script, "reset_format").unwrap();
    writeln!(
        script,
        "if confirm_prompt {}; then",
        bash_quote(check.prompt)
    )
    .unwrap();
    writeln!(script, "    points=$((points + POINTS_PER_CHECK))").unwrap();

    if let Some(fallback) = check.fallback {
        writeln!(script, "else").unwrap();
        writeln!(script, "    printf '\\n'").unwrap();
        append_display_indented(script, fallback.display, "    ");
        writeln!(script, "    reset_format").unwrap();
        writeln!(
            script,
            "    if confirm_prompt {}; then",
            bash_quote(fallback.prompt)
        )
        .unwrap();
        writeln!(script, "        points=$((points + POINTS_PER_CHECK))").unwrap();
        writeln!(script, "    fi").unwrap();
    }
    writeln!(script, "fi").unwrap();

    writeln!(script, "printf '\\n'").unwrap();
}

fn append_display(script: &mut String, display: Display) {
    append_display_indented(script, display, "");
}

fn append_display_indented(script: &mut String, display: Display, indent: &str) {
    match display {
        Display::Lines(lines) => {
            for line in lines {
                writeln!(script, "{indent}printf '%s' {}", bash_quote(line)).unwrap();
                writeln!(script, "{indent}reset_format").unwrap();
                writeln!(script, "{indent}printf '\\n'").unwrap();
            }
            writeln!(script, "{indent}printf '\\n'").unwrap();
        }
        Display::Gradient => {
            for line in GRADIENT_SCRIPT.lines() {
                writeln!(script, "{indent}{line}").unwrap();
            }
        }
    }
}

fn bash_quote(text: &str) -> String {
    let mut quoted = String::from("$'");
    for character in text.chars() {
        match character {
            '\x07' => quoted.push_str("\\a"),
            '\x1b' => quoted.push_str("\\033"),
            '\n' => quoted.push_str("\\n"),
            '\'' => quoted.push_str("\\'"),
            '\\' => quoted.push_str("\\\\"),
            character if character.is_control() => {
                write!(quoted, "\\x{:02x}", u32::from(character)).unwrap();
            }
            character => quoted.push(character),
        }
    }
    quoted.push('\'');
    quoted
}

struct TempScript {
    path: PathBuf,
}

impl TempScript {
    fn new(contents: &str) -> Result<Self, AppError> {
        let (mut file, path) = create_temp_script_file()?;
        file.write_all(contents.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(&path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions)?;
        }

        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempScript {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn create_temp_script_file() -> Result<(File, PathBuf), AppError> {
    let process_id = process::id();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut attempt = 0_u32;

    loop {
        let path = env::temp_dir().join(format!("termfactor-{process_id}-{unique}-{attempt}.sh"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                attempt = attempt.saturating_add(1);
            }
            Err(error) => return Err(AppError::from(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{create_temp_script_file, render};
    use crate::leaderboard::TerminalFeature;
    use crate::suite::MAX_SCORE;

    fn bash_condition_matches(condition: &str, points: u16) -> bool {
        if condition == "points == MAX_SCORE" {
            points == MAX_SCORE
        } else {
            let expression = condition
                .strip_prefix("points * ")
                .and_then(|value| value.split_once(" >= MAX_SCORE * "))
                .unwrap();
            let left_multiplier = expression.0.parse::<u16>().unwrap();
            let right_multiplier = expression.1.parse::<u16>().unwrap();

            u32::from(points) * u32::from(left_multiplier)
                >= u32::from(MAX_SCORE) * u32::from(right_multiplier)
        }
    }

    fn bash_feature_for_score(score: u16) -> TerminalFeature {
        let bounded_score = score.min(MAX_SCORE);

        for feature in TerminalFeature::ordered() {
            if let Some(condition) = feature.bash_condition() {
                if bash_condition_matches(&condition, bounded_score) {
                    return feature;
                }
            } else {
                return feature;
            }
        }

        TerminalFeature::Trash
    }

    #[test]
    fn generated_script_quits_on_escape() {
        let script = render();

        assert!(script.contains("$confirm == $'\\033'"));
    }

    #[test]
    fn generated_script_uses_bash_three_compatible_escape_timeouts() {
        let script = render();

        assert!(script.contains("escape_read_timeout=1"));
        assert!(script.contains("if read -r -t \"$escape_read_timeout\" -n 1 -s; then"));
        assert!(!script.contains("read -r -t 0.05"));
    }

    #[test]
    fn generated_script_does_not_reprint_prompts() {
        let script = render();

        assert!(!script.contains("\\r%s (y/n or q): "));
    }

    #[test]
    fn generated_script_uses_real_reset_escape() {
        let script = render();

        assert!(script.contains("local reset=$'\\033[0m'"));
    }

    #[test]
    fn generated_script_quits_on_eof() {
        let script = render();

        assert!(script.contains("if ! read -r -n 1 -s confirm; then"));
    }

    #[test]
    fn generated_script_does_not_assign_unused_version_variable() {
        let script = render();

        assert!(!script.contains("\nVERSION="));
    }

    #[test]
    fn bash_feature_thresholds_match_rust_at_boundaries() {
        for score in [0, 59, 60, 74, 75, 89, 90, 104, 105, 119, 120, 200] {
            assert_eq!(
                bash_feature_for_score(score),
                TerminalFeature::from_score(score)
            );
        }
    }

    #[test]
    fn generated_script_uses_case_insensitive_fastfetch_extractors() {
        let script = render();

        assert!(script.contains("tolower($0) ~ /^os:[[:space:]]*/"));
        assert!(script.contains("tolower($0) ~ /^terminal font:[[:space:]]*/"));
        assert!(script.contains("tolower($0) ~ /^terminal:[[:space:]]*/"));
    }

    #[test]
    fn generated_script_validates_system_identity_before_prompts() {
        let script = render();
        let validation_position = script
            .find("[[ -z $os_info || -z $terminal_info ]]")
            .unwrap();
        let first_prompt_position = script.find("confirm_prompt").unwrap();

        assert!(validation_position < first_prompt_position);
    }

    #[test]
    fn temp_script_files_use_unique_names() {
        let (first_file, first_path) = create_temp_script_file().unwrap();
        let (second_file, second_path) = create_temp_script_file().unwrap();

        drop(first_file);
        drop(second_file);

        assert_ne!(first_path, second_path);

        std::fs::remove_file(first_path).unwrap();
        std::fs::remove_file(second_path).unwrap();
    }
}
