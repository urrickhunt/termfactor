// TermFactor 🦀
// is your terminal sick, mid or sus?
// test terminal features & score with user responses for truecolor, text decorations, unicode, emoji, bidi, hyperlinks, notifications & sixel
// urrick hunt

#![allow(unused_imports)]
use std::env;
use std::process::{exit, Command, Stdio};
use std::io::{self, Read, Write};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

// embedded bash for terminals like mintty
static BASH_SCRIPT: &str = r#"#!/usr/bin/env bash
# termfactor.sh 🐚
# test terminal features & score with user responses for truecolor, text decorations, unicode, emoji, bidi, hyperlinks, notifications & sixel
# urrick hunt

# get the rust binary path from the first argument
RUST_BINARY_PATH="$1"
shift

trap 'echo -ne "\033[0;33;3msession terminated by user\033[0m\n"; exit 1' SIGINT SIGTERM

VERSION="0.6.9"

if [[ $1 == "--version" ]]; then
    echo "termfactor.sh $VERSION"
    exit 0
fi

if ! command -v fastfetch &> /dev/null; then
    echo "fastfetch is required but not installed. please install fastfetch & try again."
    exit 1
fi

ff_output=$(fastfetch --config none --logo none)
os_info=$(echo "$ff_output" | grep -i "OS:" | awk -F ': ' '{print $2}')
shell_info=$(basename "$SHELL")" "$($SHELL --version 2>&1 | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -n 1)
terminal_font_info=$(echo "$ff_output" | grep -i "Terminal Font:" | awk -F ': ' '{print $2}')
terminal_info=$(echo "$ff_output" | grep -i "Terminal:" | awk -F ': ' '{print $2}')

print_header() {
    local header="$1"
    echo -e "\e[1;36m# $header\e[0m"
}

reset_format() {
    printf '\x1b[0m'
}

print_header "terminal"
echo -e "OS: \033[34m$os_info\033[0m"
echo -e "shell: \e[32m$shell_info\e[0m"
echo -e "terminal font: \033[33m$terminal_font_info\033[0m"
echo -e "terminal: \e[1;35m$terminal_info\e[0m"
echo

# initialize points variable
points=0

function confirm_prompt() {
    local prompt_message="$1"
    local reset="\033[0m"
    echo -ne "$prompt_message (y/n or q): "
    while true; do
        read -n 1 -s confirm
        echo -ne "\r$prompt_message (y/n or q): "

        if [[ $confirm == [yY] ]]; then
            echo -e "\033[0;32myes${reset}"
            return 0
        elif [[ $confirm == [nN] ]]; then
            echo -e "\033[0;31mno${reset}"
            return 1
        elif [[ $confirm == [qQ] ]]; then
            echo -e "\033[0;33;3msession terminated by user${reset}"
            exit 0
        fi
        # invalid input is ignored silently
    done
}

print_header "truecolor"
printf "\x1b[38;2;255;100;0mtruecolor\x1b[0m\n"
echo

if confirm_prompt "is the word 'truecolor' displayed in $(echo -e '\033[38;2;255;100;0msafety orange\033[0m')?"; then
    points=$((points + 5))
fi
echo

awk 'BEGIN{
    s="/\\/\\/\\/\\/\\"; s=s s s s s s s s;
    for (colnum = 0; colnum<77; colnum++) {
        r = 255-(colnum*255/76);
        g = (colnum*510/76);
        b = (colnum*255/76);
        if (g>255) g = 510-g;
        printf "\033[48;2;%d;%d;%dm", r,g,b;
        printf "\033[38;2;%d;%d;%dm", 255-r,255-g,255-b;
        printf "%s\033[0m", substr(s,colnum+1,1);
    }
    printf "\n";
}'
echo

if confirm_prompt "is a $(echo -e '\033[0;31mred\033[0m')/$(echo -e '\033[0;32mgreen\033[0m')/$(echo -e '\033[0;34mblue\033[0m') color gradient displayed?"; then
    points=$((points + 5))
fi
echo

declare -A decorations=(
    ["bold"]="\e[1mbold\e[22m|is the text bold?"
    ["dim"]="\e[2mdim\e[22m|is the text dim?"
    ["italic"]="\e[3mitalic\e[23m|is the text italic?"
    ["underline"]="\e[4munderline\e[24m|is the text underlined?"
    ["double_underline"]="\e[4:2mdouble underline\e[24m|is the text double underlined?"
    ["curly_underline"]="\e[4:3mcurly underline\e[24m|is the text curly underlined?"
    ["dotted_underline"]="\e[4:4mdotted underline\e[24m|is the text dotted underlined?"
    ["dashed_underline"]="\e[4:5mdashed underline\e[24m|is the text dashed underlined?"
    ["colored_underline"]="\e[4;58:5:212mcolored underline\e[59;24m|is the text pink underlined?"
    ["blink"]="\e[5mblink\e[25m|is the text blinking?"
    ["reverse"]="\e[7mreverse\e[27m|is the text reversed?"
    ["invisible"]="\e[8minvisible\e[28m <- invisible (but copy-pasteable)|is the text invisible but still copy-pasteable?"
    ["strikethrough"]="\e[9mstrikethrough\e[29m|is the text strikethrough?"
    ["overline"]="\e[53moverline\e[55m|is the text overlined?"
)

ordered_keys=("bold" "dim" "italic" "underline" "double_underline" "curly_underline" "dotted_underline" "dashed_underline" "colored_underline" "blink" "reverse" "invisible" "strikethrough" "overline")

print_header "text decorations"

for key in "${ordered_keys[@]}"; do
    IFS='|' read -r escape_code message <<< "${decorations[$key]}"
    printf "${escape_code}\n"
    printf '\e[0m' # force reset for dumb terminals like apple terminal

    if [[ $key == "double_underline" ]]; then
        if ! confirm_prompt "$message"; then
            echo ""
            printf '\e[21malt double underline\e[24m\n'
            printf '\e[0m' # force reset for dumb terminals like apple terminal

            if confirm_prompt "is the text double underlined now?"; then
                points=$((points + 5))
            fi
        else
            points=$((points + 5))
        fi
    elif [[ $key == "dashed_underline" ]]; then
        printf '\e[0m' # force reset for dumb terminals like apple terminal
        if confirm_prompt "$message"; then
            points=$((points + 5))
        fi
    else
        if confirm_prompt "$message"; then
            points=$((points + 5))
        fi
    fi

    echo
    printf '\e[0m' # force reset for dumb terminals like apple terminal
done

print_header "unicode"
echo "Δ Й ह क あ 葉 ᄀ ቐ ๗ ഷ"
echo

if confirm_prompt "are all 10 unicode symbols displayed?"; then
    points=$((points + 5))
fi
echo

print_header "emojis"
echo "👻💜🧛"
echo

if confirm_prompt "are emojis 👻💜🧛 displayed?"; then
    points=$((points + 5))
fi
echo

print_header "bidi aware ('ע' & 'ا' symbols should be on the right side)"
echo "עברית"
echo

if confirm_prompt "is the 'ע' symbol on the right side?"; then
    points=$((points + 5))
fi
echo

echo "العربية"
echo

if confirm_prompt "is the 'ا' symbol on the right side?"; then
    points=$((points + 5))
fi
echo

print_header "osc 8 hyperlinks"
printf '\033]8;;https://github.com/Alhadis/OSC8-Adoption\033\\🔗osc 8 terminal adoption\033]8;;\033\\\n'
echo

if confirm_prompt "is a clickable hyperlink displayed (use modifier keys Ctrl+Shift if needed)?"; then
    points=$((points + 5))
fi
echo

print_header "osc 9 notifications"
printf '\e]9;✔osc 9 notifications\a'
printf '\e[5m✔check notifications\e[25m\n'
echo

if confirm_prompt "did you get the osc 9 notification?"; then
    points=$((points + 5))
fi
echo

print_header "osc 777 notifications"
printf "\e]777;notify;%s;%s\e\\" "✔osc" "777 notifications"
printf '\e[5m✔check notifications\e[25m\n'
echo

if confirm_prompt "did you get the osc 777 notification?"; then
    points=$((points + 5))
fi
echo

print_header "sixel graphics"
printf '\eP0;0;0q"1;1;64;64#0;2;0;0;0#1;2;100;100;100#1~{wo_!11?@FN^!34~^NB\n@?_ow{~$#0?BFN^!11~}wo_!34?_o{}~^NFB-#1!5~}{o_!12?BF^!25~^NB@??ow{!6~$#0!5?\n@BN^!12~{w_!25?_o{}~~NFB-#1!10~}w_!12?@BN^!15~^NFB@?_w{}!10~$#0!10?@F^!12~}\n{o_!15?_ow{}~^FB@-#1!14~}{o_!11?@BF^!7~^FB??_ow}!15~$#0!14?@BN^!11~}{w_!7?_\nw{~~^NF@-#1!18~}{wo!11?_r^FB@??ow}!20~$#0!18?@BFN!11~^K_w{}~~NF@-#1!23~M!4?\n_oWMF@!6?BN^!21~$#0!23?p!4~^Nfpw}!6~{o_-#1!18~^NB@?_ow{}~wo!12?@BFN!17~$#0!\n18?_o{}~^NFB@?FN!12~}{wo-#1!13~^NB@??_w{}!9~}{w_!12?BFN^!12~$#0!13?_o{}~~^F\nB@!9?@BF^!12~{wo_-#1!8~^NFB@?_w{}!19~{wo_!11?@BN^!8~$#0!8?_ow{}~^FB@!19?BFN\n^!11~}{o_-#1!4~^NB@?_ow{!28~}{o_!12?BF^!4~$#0!4?_o{}~^NFB!28?@BN^!12~{w_-#1\nNB@???GM!38NMG!13?@BN$#0?KMNNNF@!38?@F!13NMK-\e\\'
echo

if confirm_prompt "is a sixel graphic displayed?"; then
    points=$((points + 5))
fi
echo

print_header "score"
echo -e "OS: \033[34m$os_info\033[0m"
echo -e "terminal: \e[1;35m$terminal_info\e[0m"
percentage=$((points * 100 / 120))
echo -e "total points: \e[32m$points\e[0m/120 ($percentage%)"

if [[ $points -eq 120 ]]; then
    echo -e "\e[1;36mgoat termfactor 🐐\e[0m"
    terminal_feature="goat"
elif [[ $points -ge 105 && $points -le 119 ]]; then
    echo -e "\e[1;36msick termfactor 👑\e[0m"
    terminal_feature="sick"
elif [[ $points -ge 90 && $points -le 104 ]]; then
    echo -e "\e[1;36mlit termfactor 🔥\e[0m"
    terminal_feature="lit"
elif [[ $points -ge 75 && $points -le 89 ]]; then
    echo -e "\e[1;36mmid termfactor 😑\e[0m"
    terminal_feature="mid"
elif [[ $points -ge 60 && $points -le 74 ]]; then
    echo -e "\e[1;36msus termfactor 🤨\e[0m"
    terminal_feature="sus"
else
    echo -e "\e[1;36mtrash termfactor 🗑️\e[0m"
    terminal_feature="trash"
fi

echo
echo

# call the rust program to update terminals.txt
"$RUST_BINARY_PATH" --update-terminals "$os_info" "$terminal_info" "$points" "$terminal_feature"
"#;

fn execute_embedded_bash_script() {
    use std::process::Command;
    use std::io::Write;
    use std::fs::File;
    use std::env;

    if Command::new("bash").arg("--version").output().is_err() {
        eprintln!("bash is required but not found in PATH.");
        std::process::exit(1);
    }

    let mut temp_dir = env::temp_dir();
    temp_dir.push("interactive-testdrive.sh");

    {
        let mut file = File::create(&temp_dir).expect("failed to create temporary script file");
        file.write_all(BASH_SCRIPT.as_bytes())
            .expect("failed to write to temporary script file");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&temp_dir)
            .expect("failed to get permissions")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&temp_dir, perms).expect("failed to set permissions");
    }

    let rust_binary_path = env::current_exe().expect("failed to get current executable path");
    let rust_binary_path_str = rust_binary_path.to_str().expect("failed to convert path to string");

    let status = Command::new("bash")
        .arg(temp_dir.to_str().unwrap())
        .arg(rust_binary_path_str)
        .status()
        .expect("failed to execute bash script");

    std::fs::remove_file(&temp_dir).expect("failed to remove temporary script file");

    if !status.success() {
        eprintln!("bash script exited with status: {status}");
        std::process::exit(1);
    }
}

// rust for most terminals
#[allow(clippy::too_many_lines)]
fn main() {
    const VERSION: &str = "0.6.9";
    let args: Vec<String> = env::args().collect();

    if args.len() > 5 && args[1] == "--update-terminals" {
        let os_info = &args[2];
        let terminal_info = &args[3];
        let points: i32 = args[4].parse().unwrap_or(0);
        let terminal_feature = &args[5];

        update_terminals_txt(os_info, terminal_info, points, terminal_feature);
        std::process::exit(0);
    }

    if args.len() > 1 {
        if args[1] == "--version" {
            println!("termfactor.rs {VERSION}");
            exit(0);
        } else if args[1] == "--bash" {
            execute_embedded_bash_script();
            return;
        } else if args[1] == "--help" {
            println!("usage: termfactor [--bash]");
            println!("options:");
            println!("  --bash       run embedded bash for terminals like mintty");
            println!("  --version    show version information");
            println!("  --help       show this help message");
            exit(0);
        }
    }

    let mut points = 0;

    check_fastfetch_installed();

    let (os_info, shell_info, terminal_info, terminal_font_info) = get_system_info();

    print_header("terminal");
    println!("OS: \x1b[34m{os_info}\x1b[0m");
    println!("shell: \x1b[32m{shell_info}\x1b[0m");
    println!("terminal font: \x1b[33m{terminal_font_info}\x1b[0m");
    println!("terminal: \x1b[1;35m{terminal_info}\x1b[0m\n");

    print_header("truecolor");
    println!("\x1b[38;2;255;100;0mtruecolor\x1b[0m\n");
    if confirm_prompt("is the word 'truecolor' displayed in \x1b[38;2;255;100;0msafety orange\x1b[0m?") {
        points += 5;
    }
    println!();

    print_color_gradient();
    if confirm_prompt("is a \x1b[0;31mred\x1b[0m/\x1b[0;32mgreen\x1b[0m/\x1b[0;34mblue\x1b[0m color gradient displayed?") {
        points += 5;
    }
    println!();

    print_header("text decorations");
    print_text_decorations(&mut points);

    print_header("unicode");
    println!("Δ Й ह क あ 葉 ᄀ ቐ ๗ ഷ\n");
    if confirm_prompt("are all 10 unicode symbols displayed?") {
        points += 5;
    }
    println!();

    print_header("emojis");
    println!("👻💜🧛\n");
    if confirm_prompt("are emojis 👻💜🧛 displayed?") {
        points += 5;
    }
    println!();

    print_header("bidi aware ('ע' & 'ا' symbols should be on the right side)");
    println!("עברית\n");
    if confirm_prompt("is the 'ע' symbol on the right side?") {
        points += 5;
    }
    println!();

    println!("العربية\n");
    if confirm_prompt("is the 'ا' symbol on the right side?") {
        points += 5;
    }
    println!();

    print_header("osc 8 hyperlinks");
    println!("\x1b]8;;https://github.com/Alhadis/OSC8-Adoption\x1b\\🔗osc 8 terminal adoption\x1b]8;;\x1b\\\n");
    if confirm_prompt("is a clickable hyperlink displayed (use modifier keys ctrl+shift if needed)?") {
        points += 5;
    }
    println!();

    print_header("osc 9 notifications");
    print!("\x1b]9;✔osc 9 notifications\x07");
    println!("\x1b[5m✔check notifications\x1b[25m");
    println!();
    if confirm_prompt("did you get the osc 9 notification?") {
        points += 5;
    }
    println!();

    print_header("osc 777 notifications");
    print!("\x1b]777;notify;✔osc;777 notifications\x1b\\");
    println!("\x1b[5m✔check notifications\x1b[25m");
    println!();
    if confirm_prompt("did you get the osc 777 notification?") {
        points += 5;
    }
    println!();

    print_header("sixel graphics");
    print_sixel_graphics();
    if confirm_prompt("is a sixel graphic displayed?") {
        points += 5;
    }
    println!();

    print_header("score");
    println!("OS: \x1b[34m{os_info}\x1b[0m");
    println!("terminal: \x1b[1;35m{terminal_info}\x1b[0m");
    let percentage = (points * 100) / 120;
    println!("total points: \x1b[32m{points}\x1b[0m/120 ({percentage}%)");

    let terminal_feature = if points == 120 {
        println!("\x1b[1;36mgoat termfactor 🐐\x1b[0m");
        "goat"
    } else if (105..=119).contains(&points) {
        println!("\x1b[1;36msick termfactor 👑\x1b[0m");
        "sick"
    } else if (90..=104).contains(&points) {
        println!("\x1b[1;36mlit termfactor 🔥\x1b[0m");
        "lit"
    } else if (75..=89).contains(&points) {
        println!("\x1b[1;36mmid termfactor 😑\x1b[0m");
        "mid"
    } else if (60..=74).contains(&points) {
        println!("\x1b[1;36msus termfactor 🤨\x1b[0m");
        "sus"
    } else {
        println!("\x1b[1;36mtrash termfactor 🗑️\x1b[0m");
        "trash"
    };

    println!();
    println!();

    update_terminals_txt(&os_info, &terminal_info, points, terminal_feature);
}

fn check_fastfetch_installed() {
    let output = Command::new("fastfetch")
        .arg("--version")
        .output();
    match output {
        Ok(output) if output.status.success() => {}
        _ => {
            println!("fastfetch is required but not installed. please install fastfetch & try again.");
            exit(1);
        }
    }
}

fn run_command(command: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", command])
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
    };

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(output) => {
            println!("command failed: {command}");
            println!("{}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        Err(_) => {
            println!("failed to execute command: {command}");
            exit(1);
        }
    }
}

fn get_system_info() -> (String, String, String, String) {
    let ff_output = run_command("fastfetch --config none --logo none");

    let mut os_info = String::new();
    let shell_info;
    let mut terminal_info = String::new();
    let mut terminal_font_info = String::new();

    for line in ff_output.lines() {
        if line.to_lowercase().starts_with("os:") {
            os_info = line.split(": ").nth(1).unwrap_or("").to_string();
        } else if line.to_lowercase().starts_with("terminal font:") {
            terminal_font_info = line.split(": ").nth(1).unwrap_or("").to_string();
        } else if line.to_lowercase().starts_with("terminal:") {
            terminal_info = line.split(": ").nth(1).unwrap_or("").to_string();
        }
    }

    if let Ok(shell_path) = env::var("SHELL") {
        let default_shell = shell_path.split('/').last().unwrap_or("").to_string();

        let output = Command::new(&shell_path)
            .arg("--version")
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let re = Regex::new(r"(\d+\.\d+(\.\d+)?)").unwrap();
            let shell_version = if let Some(cap) = re.captures(&stdout) {
                cap.get(1).map_or("unknown version", |m| m.as_str())
            } else {
                "unknown version"
            };
            shell_info = format!("{default_shell} {shell_version}");
        } else {
            shell_info = format!("{default_shell} unknown version");
        }

    } else {
        // try pwsh or powershell
        let output = Command::new("pwsh")
            .args(["-Command", "$PSVersionTable.PSVersion.ToString()"])
            .output();

        if let Ok(output) = output {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if version.is_empty() {
                shell_info = "pwsh unknown version".to_string();
            } else {
                shell_info = format!("pwsh {version}");
            }
        } else {
            let output = Command::new("powershell")
                .args(["-Command", "$PSVersionTable.PSVersion.ToString()"])
                .output();

            if let Ok(output) = output {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if version.is_empty() {
                    shell_info = "powershell unknown version".to_string();
                } else {
                    shell_info = format!("powershell {version}");
                }
            } else {
                shell_info = "powershell unknown version".to_string();
            }
        }
    }

    (os_info, shell_info, terminal_info, terminal_font_info)
}

fn print_header(header: &str) {
    println!("\x1b[1;36m# {header}\x1b[0m");
}

fn reset_format() {
    print!("\x1b[0m");
    io::stdout().flush().unwrap();
}

fn confirm_prompt(prompt_message: &str) -> bool {
    let reset = "\x1b[0m";
    print!("{prompt_message} (y/n or q): ");
    io::stdout().flush().unwrap();

    loop {
        let confirm = read_single_key();
        match confirm.to_ascii_lowercase() {
            'y' => {
                println!("\x1b[0;32myes{reset}");
                return true;
            }
            'n' => {
                println!("\x1b[0;31mno{reset}");
                return false;
            }
            'q' => {
                println!("\x1b[0;33;3msession terminated by user{reset}");
                exit(0);
            }
            _ => {
                // ignore other keys
            }
        }
    }
}

fn read_single_key() -> char {
    #[cfg(unix)]
    {
        use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg};
        use std::os::unix::io::AsRawFd;
        let stdin_fd = io::stdin().as_raw_fd();

        let orig_termios = tcgetattr(stdin_fd).unwrap();
        let mut raw = orig_termios.clone();
        raw.local_flags.remove(LocalFlags::ICANON | LocalFlags::ECHO);
        tcsetattr(stdin_fd, SetArg::TCSANOW, &raw).unwrap();

        let mut buf = [0u8; 1];
        let res = io::stdin().read_exact(&mut buf);

        tcsetattr(stdin_fd, SetArg::TCSANOW, &orig_termios).unwrap();

        match res {
            Ok(()) => buf[0] as char,
            Err(_) => '\0',
        }
    }

    #[cfg(windows)]
    {
        use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
        use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT};
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_INPUT_HANDLE;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::shared::minwindef::DWORD;
        use std::io::Read;

        unsafe {
            let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
            assert!(h_stdin != INVALID_HANDLE_VALUE, "failed to get stdin handle");

            let mut mode: DWORD = 0;
            assert!(GetConsoleMode(h_stdin, &mut mode) != 0, "failed to get console mode");

            let original_mode = mode;

            mode &= !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT);
            assert!(SetConsoleMode(h_stdin, mode) != 0, "failed to set console mode");

            let mut buf = [0u8; 1];
            let res = io::stdin().read_exact(&mut buf);

            SetConsoleMode(h_stdin, original_mode);

            match res {
                Ok(()) => buf[0] as char,
                Err(_) => '\0',
            }
        }
    }
}

fn print_color_gradient() {
    let s = "/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/\\/";
    let length = s.len();
    for colnum in 0..77 {
        let r = 255 - (colnum * 255 / 76);
        let mut g = colnum * 510 / 76;
        let b = colnum * 255 / 76;
        if g > 255 {
            g = 510 - g;
        }
        print!(
            "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}",
            r,
            g,
            b,
            255 - r,
            255 - g,
            255 - b,
            s.chars().nth(colnum % length).unwrap()
        );
    }
    println!("\x1b[0m\n");
}

fn print_text_decorations(points: &mut i32) {
    use std::collections::HashMap;
    let mut decorations: HashMap<&str, (&str, &str)> = HashMap::new();
    decorations.insert("bold", ("\x1b[1mbold\x1b[22m", "is the text bold?"));
    decorations.insert("dim", ("\x1b[2mdim\x1b[22m", "is the text dim?"));
    decorations.insert("italic", ("\x1b[3mitalic\x1b[23m", "is the text italic?"));
    decorations.insert("underline", ("\x1b[4munderline\x1b[24m", "is the text underlined?"));
    decorations.insert(
        "double_underline",
        (
            "\x1b[4:2mdouble underline\x1b[24m",
            "is the text double underlined?",
        ),
    );
    decorations.insert(
        "curly_underline",
        (
            "\x1b[4:3mcurly underline\x1b[24m",
            "is the text curly underlined?",
        ),
    );
    decorations.insert(
        "dotted_underline",
        (
            "\x1b[4:4mdotted underline\x1b[24m",
            "is the text dotted underlined?",
        ),
    );
    decorations.insert(
        "dashed_underline",
        (
            "\x1b[4:5mdashed underline\x1b[24m",
            "is the text dashed underlined?",
        ),
    );
    decorations.insert(
        "colored_underline",
        (
            "\x1b[4;58:5:212mcolored underline\x1b[59;24m",
            "is the text pink underlined?",
        ),
    );
    decorations.insert("blink", ("\x1b[5mblink\x1b[25m", "is the text blinking?"));
    decorations.insert("reverse", ("\x1b[7mreverse\x1b[27m", "is the text reversed?"));
    decorations.insert(
        "invisible",
        (
            "\x1b[8minvisible\x1b[28m <- invisible (but copy-pasteable)",
            "is the text invisible but still copy-pasteable?",
        ),
    );
    decorations.insert(
        "strikethrough",
        ("\x1b[9mstrikethrough\x1b[29m", "is the text strikethrough?"),
    );
    decorations.insert("overline", ("\x1b[53moverline\x1b[55m", "is the text overlined?"));

    let ordered_keys = vec![
        "bold",
        "dim",
        "italic",
        "underline",
        "double_underline",
        "curly_underline",
        "dotted_underline",
        "dashed_underline",
        "colored_underline",
        "blink",
        "reverse",
        "invisible",
        "strikethrough",
        "overline",
    ];

    for key in ordered_keys {
        let (escape_code, message) = decorations[key];
        println!("{escape_code}");
        reset_format();

        if key == "double_underline" {
            if confirm_prompt(message) {
                *points += 5;
            } else {
                println!();
                println!("\x1b[21malt double underline\x1b[24m");
                reset_format();
                if confirm_prompt("is the text double underlined now?") {
                    *points += 5;
                }
            }
        } else if key == "dashed_underline" {
            reset_format(); // force reset for dumb terminals like apple terminal
            if confirm_prompt(message) {
                *points += 5;
            }
        } else if confirm_prompt(message) {
            *points += 5;
        }

        println!();
        reset_format();
    }
}

fn print_sixel_graphics() {
    let sixel_data = "\x1bP0;0;0q\"1;1;64;64#0;2;0;0;0#1;2;100;100;100#1~{wo_!11?@FN^!34~^NB\
        @?_ow{~$#0?BFN^!11~}wo_!34?_o{}~^NFB-#1!5~}{o_!12?BF^!25~^NB@??ow{!6~$#0!5?\
        @BN^!12~{w_!25?_o{}~~NFB-#1!10~}w_!12?@BN^!15~^NFB@?_w{}!10~$#0!10?@F^!12~}\
        {o_!15?_ow{}~^FB@-#1!14~}{o_!11?@BF^!7~^FB??_ow}!15~$#0!14?@BN^!11~}{w_!7?_\
        w{~~^NF@-#1!18~}{wo!11?_r^FB@??ow}!20~$#0!18?@BFN!11~^K_w{}~~NF@-#1!23~M!4?_\
        oWMF@!6?BN^!21~$#0!23?p!4~^Nfpw}!6~{o_-#1!18~^NB@?_ow{}~wo!12?@BFN!17~$#0!18?_\
        o{}~^NFB@?FN!12~}{wo-#1!13~^NB@??_w{}!9~}{w_!12?BFN^!12~$#0!13?_o{}~~^FB@!9?\
        @BF^!12~{wo_-#1!8~^NFB@?_w{}!19~{wo_!11?@BN^!8~$#0!8?_ow{}~^FB@!19?BFN^!11~}\
        {o_-#1!4~^NB@?_ow{!28~}{o_!12?BF^!4~$#0!4?_o{}~^NFB!28?@BN^!12~{w_-#1NB@???GM\
        !38NMG!13?@BN$#0?KMNNNF@!38?@F!13NMK-\x1b\\";
    print!("{sixel_data}");
    println!();
}

#[allow(clippy::too_many_lines)]
fn update_terminals_txt(
    os_info: &str,
    terminal_info: &str,
    points: i32,
    terminal_feature: &str,
) {
    use std::cmp::Ordering;

    let file_path = "terminals.txt";
    let mut entries = Vec::new();

    if Path::new(file_path).exists() {
        let file = File::open(file_path).expect("failed to open terminals.txt");
        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if line.len() < 2 {
                continue;
            }

            let number = &line[0..2].trim().to_string();
            let score = &line[4..11].trim().to_string();
            let feature = &line[13..18].trim().to_string();
            let terminal = &line[20..56].trim().to_string();
            let os = if line.len() >= 104 {
                line[58..106].trim().to_string()
            } else if line.len() > 58 {
                line[58..].trim().to_string()
            } else {
                String::new()
            };
            entries.push((
                number.to_string(),
                score.to_string(),
                feature.to_string(),
                terminal.to_string(),
                os.to_string(),
            ));
        }
    }

    let mut updated = false;

    let base_terminal_info = extract_base_terminal_name(terminal_info);
    let base_os_info = extract_base_os_name(os_info);

    for entry in &mut entries {
        let entry_base_terminal = extract_base_terminal_name(&entry.3);
        let entry_base_os = extract_base_os_name(&entry.4);

        if entry_base_terminal == base_terminal_info && entry_base_os == base_os_info {
            // compare OS versions
            let entry_os_version = extract_os_version(&entry.4);
            let new_os_version = extract_os_version(os_info);

            match new_os_version.cmp(&entry_os_version) {
                Ordering::Greater => {
                    // replace with the new entry
                    entry.1 = format!("{points}/120");
                    entry.2 = terminal_feature.to_string();
                    entry.3 = terminal_info.to_string();
                    entry.4 = os_info.to_string();
                }
                Ordering::Equal => {
                    // same OS version, update if points are higher
                    let existing_points: i32 = entry
                        .1
                        .split('/')
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap_or(0);
                    if points > existing_points {
                        entry.1 = format!("{points}/120");
                        entry.2 = terminal_feature.to_string();
                    }
                }
                Ordering::Less => {
                    // do nothing, keep the existing entry
                }
            }
            updated = true;
            break;
        }
    }

    if !updated {
        entries.push((
            String::new(),
            format!("{points}/120"),
            terminal_feature.to_string(),
            terminal_info.to_string(),
            os_info.to_string(),
        ));
    }

    // sort entries
    entries.sort_by(|a, b| {
        let a_terminal = extract_base_terminal_name(&a.3);
        let b_terminal = extract_base_terminal_name(&b.3);
        let a_os = extract_base_os_name(&a.4);
        let b_os = extract_base_os_name(&b.4);

        if a_terminal == b_terminal && a_os == b_os {
            // compare OS version (descending)
            let a_os_version = extract_os_version(&a.4);
            let b_os_version = extract_os_version(&b.4);
            b_os_version
                .cmp(&a_os_version)
                .then_with(|| {
                    // compare points (descending)
                    let a_points: i32 = a.1.split('/').next().unwrap().parse().unwrap_or(0);
                    let b_points: i32 = b.1.split('/').next().unwrap().parse().unwrap_or(0);
                    b_points.cmp(&a_points)
                })
        } else {
            a_terminal
                .cmp(&b_terminal)
                .then_with(|| a_os.cmp(&b_os))
                .then_with(|| {
                    // compare OS version (descending)
                    let a_os_version = extract_os_version(&a.4);
                    let b_os_version = extract_os_version(&b.4);
                    b_os_version.cmp(&a_os_version)
                })
                .then_with(|| {
                    // compare points (descending)
                    let a_points: i32 = a.1.split('/').next().unwrap().parse().unwrap_or(0);
                    let b_points: i32 = b.1.split('/').next().unwrap().parse().unwrap_or(0);
                    b_points.cmp(&a_points)
                })
        }
    });

    // deduplicate entries
    entries.dedup_by(|a, b| {
        extract_base_terminal_name(&a.3) == extract_base_terminal_name(&b.3)
            && extract_base_os_name(&a.4) == extract_base_os_name(&b.4)
    });

    // sort by points descending
    entries.sort_by(|a, b| {
        let a_points: i32 = a.1.split('/').next().unwrap().parse().unwrap_or(0);
        let b_points: i32 = b.1.split('/').next().unwrap().parse().unwrap_or(0);
        b_points.cmp(&a_points)
    });

    if entries.len() > 25 {
        entries.truncate(25);
    }

    for (i, entry) in entries.iter_mut().enumerate() {
        entry.0 = format!("{:02}", i + 1);
    }

    let file = File::create(file_path).expect("failed to create terminals.txt");
    let mut writer = BufWriter::new(file);

    writeln!(writer, "# term factors\n").unwrap();
    for entry in &entries {
        writeln!(
            writer,
            "{:<2}  {:>7}  {:<5}  {:<36}  {:<48}",
            entry.0, entry.1, entry.2, entry.3, entry.4
        )
        .unwrap();
    }

    println!("\x1b[1;36m# termfactors\x1b[0m");
    for entry in &entries {
        let parts: Vec<&str> = entry.1.split('/').collect();
        let numerator = parts.first().copied().unwrap_or("");
        let denominator = parts.get(1).copied().unwrap_or("");

        let numerator_padded = format!("{:03}", numerator.parse::<i32>().unwrap_or(0));

        let score_display = format!("\x1b[32m{numerator_padded}\x1b[0m/{denominator}");

        println!(
            "{:<2}  {}  \x1b[1;36m{:<5}\x1b[0m  \x1b[1;35m{:<36}\x1b[0m  \x1b[34m{:<48}\x1b[0m",
            entry.0, score_display, entry.2, entry.3, entry.4
        );
    }
}

fn extract_base_terminal_name(terminal_info: &str) -> String {
    let mut parts: Vec<&str> = terminal_info.split_whitespace().collect();
    while let Some(&last_part) = parts.last() {
        let first_char = last_part.chars().next().unwrap_or('\0');
        if first_char == 'v' || first_char.is_ascii_digit() {
            parts.pop();
        } else {
            break;
        }
    }
    parts.join(" ")
}

fn extract_base_os_name(os_info: &str) -> String {
    let mut base_os_name = String::new();
    for part in os_info.split_whitespace() {
        if part.chars().next().unwrap_or(' ').is_ascii_digit() {
            break;
        }
        if !base_os_name.is_empty() {
            base_os_name.push(' ');
        }
        base_os_name.push_str(part);
    }
    base_os_name
}

fn extract_os_version(os_info: &str) -> String {
    for part in os_info.split_whitespace() {
        if part.chars().next().unwrap_or(' ').is_ascii_digit() {
            return part.to_string();
        }
    }
    String::new()
}

