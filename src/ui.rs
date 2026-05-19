use std::io::{self, ErrorKind, Write};

use crate::error::AppError;
use crate::leaderboard::TerminalFeature;
use crate::platform::read_single_key;
use crate::suite::{Check, Display, CHECKS, MAX_SCORE, POINTS_PER_CHECK};
use crate::system::SystemInfo;

pub struct RunSummary {
    pub score: u16,
    pub feature: TerminalFeature,
}

pub fn run(system_info: &SystemInfo) -> Result<RunSummary, AppError> {
    print_system_info(system_info);

    let mut score = 0_u16;
    let mut previous_header = None;

    for check in CHECKS {
        run_check(check, &mut previous_header, &mut score)?;
    }

    print_summary(system_info, score);

    Ok(RunSummary {
        score,
        feature: TerminalFeature::from_score(score),
    })
}

fn print_system_info(system_info: &SystemInfo) {
    print_header("terminal");
    println!("OS: \x1b[34m{}\x1b[0m", system_info.os);
    println!("shell: \x1b[32m{}\x1b[0m", system_info.shell);
    println!(
        "terminal font: \x1b[33m{}\x1b[0m",
        system_info.terminal_font
    );
    println!("terminal: \x1b[1;35m{}\x1b[0m\n", system_info.terminal);
}

fn run_check(
    check: Check,
    previous_header: &mut Option<&'static str>,
    score: &mut u16,
) -> Result<(), AppError> {
    if *previous_header != Some(check.header) {
        print_header(check.header);
        *previous_header = Some(check.header);
    }

    render_display(check.display)?;

    if confirm(check.prompt)? {
        *score += POINTS_PER_CHECK;
    } else if let Some(fallback) = check.fallback {
        println!();
        render_display(fallback.display)?;
        if confirm(fallback.prompt)? {
            *score += POINTS_PER_CHECK;
        }
    }

    println!();
    Ok(())
}

fn render_display(display: Display) -> Result<(), AppError> {
    match display {
        Display::Lines(lines) => {
            for line in lines {
                print!("{line}");
                reset_format()?;
                println!();
            }
            println!();
        }
        Display::Gradient => {
            print_color_gradient();
            reset_format()?;
            println!();
            println!();
        }
    }
    Ok(())
}

fn print_summary(system_info: &SystemInfo, score: u16) {
    print_header("score");
    println!("OS: \x1b[34m{}\x1b[0m", system_info.os);
    println!("terminal: \x1b[1;35m{}\x1b[0m", system_info.terminal);
    println!(
        "total points: \x1b[32m{score}\x1b[0m/{MAX_SCORE} ({}%)",
        (score * 100) / MAX_SCORE
    );
    println!(
        "\x1b[1;36m{}\x1b[0m",
        TerminalFeature::from_score(score).banner()
    );
    println!();
    println!();
}

fn print_header(header: &str) {
    println!("\x1b[1;36m# {header}\x1b[0m");
}

fn confirm(prompt: &str) -> Result<bool, AppError> {
    print!("{prompt} (y/n or q): ");
    io::stdout().flush()?;

    loop {
        let key = match read_single_key() {
            Ok(key) => key,
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
                println!("\x1b[0;33;3msession terminated by user\x1b[0m");
                return Err(AppError::Quit);
            }
            Err(error) => return Err(AppError::from(error)),
        };

        match key.to_ascii_lowercase() {
            'y' => {
                println!("\x1b[0;32myes\x1b[0m");
                return Ok(true);
            }
            'n' => {
                println!("\x1b[0;31mno\x1b[0m");
                return Ok(false);
            }
            'q' | '\u{3}' | '\u{4}' | '\u{1b}' => {
                println!("\x1b[0;33;3msession terminated by user\x1b[0m");
                return Err(AppError::Quit);
            }
            _ => {}
        }
    }
}

fn print_color_gradient() {
    for column in 0..77 {
        let red = 255 - (column * 255 / 76);
        let mut green = column * 510 / 76;
        let blue = column * 255 / 76;
        if green > 255 {
            green = 510 - green;
        }

        let symbol = if column % 2 == 0 { '/' } else { '\\' };
        print!(
            "\x1b[48;2;{red};{green};{blue}m\x1b[38;2;{};{};{}m{symbol}",
            255 - red,
            255 - green,
            255 - blue,
        );
    }
}

fn reset_format() -> Result<(), AppError> {
    print!("\x1b[0m");
    io::stdout().flush()?;
    Ok(())
}
