use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use unicode_width::UnicodeWidthStr;

use crate::error::AppError;
use crate::paths::{leaderboard_path, legacy_leaderboard_path};
use crate::suite::MAX_SCORE;

const MAX_ENTRIES: usize = 25;
const STORAGE_SEPARATOR: char = '\t';
const LEGACY_TERMINAL_WIDTH: usize = 36;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalFeature {
    Goat,
    Sick,
    Lit,
    Mid,
    Sus,
    Trash,
}

#[derive(Clone, Copy)]
struct FeatureThreshold {
    left_multiplier: u16,
    right_multiplier: u16,
    requires_exact_max_score: bool,
}

impl FeatureThreshold {
    const fn exact_max_score() -> Self {
        Self {
            left_multiplier: 1,
            right_multiplier: 1,
            requires_exact_max_score: true,
        }
    }

    const fn ratio(left_multiplier: u16, right_multiplier: u16) -> Self {
        Self {
            left_multiplier,
            right_multiplier,
            requires_exact_max_score: false,
        }
    }

    const fn matches(self, score: u16) -> bool {
        let bounded_score = if score > MAX_SCORE { MAX_SCORE } else { score } as u32;
        let max_score = MAX_SCORE as u32;

        if self.requires_exact_max_score {
            bounded_score == max_score
        } else {
            bounded_score * self.left_multiplier as u32 >= max_score * self.right_multiplier as u32
        }
    }

    fn bash_condition(self) -> String {
        if self.requires_exact_max_score {
            "points == MAX_SCORE".to_string()
        } else {
            format!(
                "points * {} >= MAX_SCORE * {}",
                self.left_multiplier, self.right_multiplier
            )
        }
    }
}

impl TerminalFeature {
    const ORDERED: [Self; 6] = [
        Self::Goat,
        Self::Sick,
        Self::Lit,
        Self::Mid,
        Self::Sus,
        Self::Trash,
    ];

    pub const fn from_score(score: u16) -> Self {
        let mut index = 0;

        while index < Self::ORDERED.len() {
            let feature = Self::ORDERED[index];
            if let Some(threshold) = feature.threshold() {
                if threshold.matches(score) {
                    return feature;
                }
            }
            index += 1;
        }

        Self::Trash
    }

    pub const fn ordered() -> [Self; 6] {
        Self::ORDERED
    }

    const fn threshold(self) -> Option<FeatureThreshold> {
        match self {
            Self::Goat => Some(FeatureThreshold::exact_max_score()),
            Self::Sick => Some(FeatureThreshold::ratio(8, 7)),
            Self::Lit => Some(FeatureThreshold::ratio(4, 3)),
            Self::Mid => Some(FeatureThreshold::ratio(8, 5)),
            Self::Sus => Some(FeatureThreshold::ratio(2, 1)),
            Self::Trash => None,
        }
    }

    pub fn bash_condition(self) -> Option<String> {
        self.threshold().map(FeatureThreshold::bash_condition)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Goat => "goat",
            Self::Sick => "sick",
            Self::Lit => "lit",
            Self::Mid => "mid",
            Self::Sus => "sus",
            Self::Trash => "trash",
        }
    }

    pub const fn banner(self) -> &'static str {
        match self {
            Self::Goat => "goat termfactor 🐐",
            Self::Sick => "sick termfactor 👑",
            Self::Lit => "lit termfactor 🔥",
            Self::Mid => "mid termfactor 😑",
            Self::Sus => "sus termfactor 🤨",
            Self::Trash => "trash termfactor 🗑️",
        }
    }
}

impl Display for TerminalFeature {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.pad(self.as_str())
    }
}

impl FromStr for TerminalFeature {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "goat" => Ok(Self::Goat),
            "sick" => Ok(Self::Sick),
            "lit" => Ok(Self::Lit),
            "mid" => Ok(Self::Mid),
            "sus" => Ok(Self::Sus),
            "trash" => Ok(Self::Trash),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LeaderboardEntry {
    rank: usize,
    score: u16,
    feature: TerminalFeature,
    terminal: String,
    os: String,
}

#[derive(Default)]
struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

pub fn update_file(
    os: &str,
    terminal: &str,
    score: u16,
    feature: TerminalFeature,
) -> Result<(), AppError> {
    validate_identity(os, terminal)?;

    let path = leaderboard_path();
    let legacy_path = legacy_leaderboard_path();
    let mut leaderboard = load_leaderboard(&path, &legacy_path)?;
    leaderboard.record(LeaderboardEntry::new(score, feature, terminal, os));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    leaderboard.write(&path)?;
    leaderboard.print(&path);
    Ok(())
}

pub fn validate_identity(os: &str, terminal: &str) -> Result<(), AppError> {
    validate_storage_field("terminal", terminal)?;
    validate_storage_field("os", os)
}

fn load_leaderboard(preferred_path: &Path, legacy_path: &Path) -> Result<Leaderboard, AppError> {
    if let Some(leaderboard) = Leaderboard::load_existing(preferred_path)? {
        if !leaderboard.entries.is_empty() || preferred_path == legacy_path {
            return Ok(leaderboard);
        }
    }

    if preferred_path == legacy_path {
        Ok(Leaderboard::default())
    } else {
        Ok(Leaderboard::load_existing(legacy_path)?.unwrap_or_default())
    }
}

impl Leaderboard {
    fn load_existing(path: &Path) -> Result<Option<Self>, AppError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(AppError::from(error)),
        };
        let mut leaderboard = Self::default();

        for line in BufReader::new(file).lines() {
            let line = line?;
            if let Some(entry) = LeaderboardEntry::parse(&line) {
                leaderboard.record(entry);
            }
        }

        Ok(Some(leaderboard))
    }

    fn record(&mut self, candidate: LeaderboardEntry) {
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|entry| entry.matches_family(&candidate))
        {
            if candidate.is_better_than(existing) {
                *existing = candidate;
            }
        } else {
            self.entries.push(candidate);
        }

        self.entries
            .sort_unstable_by(LeaderboardEntry::display_order);
        self.entries.truncate(MAX_ENTRIES);

        for (index, entry) in self.entries.iter_mut().enumerate() {
            entry.rank = index + 1;
        }
    }

    fn write(&self, path: &Path) -> Result<(), AppError> {
        let (file, temp_path) = create_temporary_write_file(path)?;
        let result = (|| -> Result<(), AppError> {
            let mut writer = BufWriter::new(file);

            writeln!(writer, "# term factors\n")?;
            for entry in &self.entries {
                validate_storage_field("terminal", &entry.terminal)?;
                validate_storage_field("os", &entry.os)?;
                writeln!(
                    writer,
                    "{:02}\t{}/{}\t{}\t{}\t{}",
                    entry.rank,
                    entry.score,
                    MAX_SCORE,
                    entry.feature.as_str(),
                    entry.terminal,
                    entry.os,
                )?;
            }

            let file = writer
                .into_inner()
                .map_err(|error| AppError::from(error.into_error()))?;
            file.sync_all()?;

            Ok(())
        })();

        if let Err(error) = result {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }

        let rename_result = fs::rename(&temp_path, path).map_err(AppError::from);
        if rename_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }

        rename_result
    }

    fn print(&self, path: &Path) {
        println!("\x1b[1;36m# termfactors\x1b[0m");
        let terminal_width = self
            .entries
            .iter()
            .map(|entry| display_width(&entry.terminal))
            .max()
            .unwrap_or_default();

        for entry in &self.entries {
            println!("{}", entry.console_row(terminal_width));
        }
        println!("\x1b[2msaved to {}\x1b[0m", path.display());
    }
}

impl LeaderboardEntry {
    fn new(score: u16, feature: TerminalFeature, terminal: &str, os: &str) -> Self {
        Self {
            rank: 0,
            score,
            feature,
            terminal: terminal.to_string(),
            os: os.to_string(),
        }
    }

    fn parse(line: &str) -> Option<Self> {
        if line.starts_with('#') || line.trim().is_empty() {
            return None;
        }

        let [rank, score, feature, terminal, os] = split_columns(line)?;
        let rank = rank.parse::<usize>().ok()?;
        let score = parse_score(score)?;
        let feature = feature
            .parse::<TerminalFeature>()
            .unwrap_or_else(|()| TerminalFeature::from_score(score));
        let terminal = terminal.to_string();
        let os = os.to_string();

        if terminal.trim().is_empty() || os.trim().is_empty() {
            return None;
        }

        Some(Self {
            rank,
            score,
            feature,
            terminal,
            os,
        })
    }

    fn matches_family(&self, other: &Self) -> bool {
        same_key_family(&self.terminal_key(), &other.terminal_key())
            && same_key_family(&self.os_key(), &other.os_key())
    }

    fn is_better_than(&self, other: &Self) -> bool {
        self.score
            .cmp(&other.score)
            .then_with(|| self.os_version().cmp(&other.os_version()))
            .then_with(|| self.data_quality().cmp(&other.data_quality()))
            .is_gt()
    }

    fn display_order(left: &Self, right: &Self) -> Ordering {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.terminal_key().cmp(&right.terminal_key()))
            .then_with(|| left.os_key().cmp(&right.os_key()))
            .then_with(|| right.os_version().cmp(&left.os_version()))
            .then_with(|| left.terminal.cmp(&right.terminal))
            .then_with(|| left.os.cmp(&right.os))
    }

    fn terminal_key(&self) -> String {
        base_terminal_name(&self.terminal)
    }

    fn os_key(&self) -> String {
        base_os_name(&self.os)
    }

    fn os_version(&self) -> VersionKey {
        VersionKey::from_os(&self.os)
    }

    fn data_quality(&self) -> usize {
        self.terminal.chars().count() + self.os.chars().count()
    }

    fn console_row(&self, terminal_width: usize) -> String {
        let padded_terminal = pad_display_width(&self.terminal, terminal_width);
        format!(
            "{:<2}  \x1b[32m{:03}\x1b[0m/{MAX_SCORE}  \x1b[1;36m{:<5}\x1b[0m  \x1b[1;35m{}\x1b[0m  \x1b[34m{}\x1b[0m",
            self.rank,
            self.score,
            self.feature,
            padded_terminal,
            self.os,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VersionKey(Vec<u32>);

impl VersionKey {
    fn from_os(os: &str) -> Self {
        let version_token = os
            .split_whitespace()
            .find(|part| starts_with_ascii_digit(part))
            .unwrap_or("");

        Self(parse_number_groups(version_token))
    }
}

impl Ord for VersionKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let max_length = self.0.len().max(other.0.len());

        for index in 0..max_length {
            let left = self.0.get(index).copied().unwrap_or(0);
            let right = other.0.get(index).copied().unwrap_or(0);

            match left.cmp(&right) {
                Ordering::Equal => {}
                ordering => return ordering,
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for VersionKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_score(value: &str) -> Option<u16> {
    value
        .split_once('/')
        .map_or(value, |(score, _)| score)
        .trim()
        .parse::<u16>()
        .ok()
}

fn split_columns(line: &str) -> Option<[&str; 5]> {
    if line.contains(STORAGE_SEPARATOR) {
        split_tab_separated_columns(line)
    } else {
        split_legacy_columns(line)
    }
}

fn split_tab_separated_columns(line: &str) -> Option<[&str; 5]> {
    let mut fields = line.split(STORAGE_SEPARATOR);
    let rank = fields.next()?.trim();
    let score = fields.next()?.trim();
    let feature = fields.next()?.trim();
    let terminal = fields.next()?;
    let os = fields.next()?;

    match (fields.next(), fields.next()) {
        (None, _) | (Some(""), None) => Some([rank, score, feature, terminal, os]),
        _ => None,
    }
}

fn split_legacy_columns(line: &str) -> Option<[&str; 5]> {
    let line = line.trim_end();
    let (rank, remainder) = take_token(line)?;
    let (score, remainder) = take_token(remainder)?;
    let (feature, remainder) = take_token(remainder)?;
    let tail = remainder.trim_start_matches(' ');
    let (terminal, os) = split_legacy_tail(tail)?;

    Some([rank, score, feature, terminal, os])
}

fn take_token(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start_matches(' ');
    if text.is_empty() {
        return None;
    }

    let end = text.find(' ').unwrap_or(text.len());
    Some((&text[..end], &text[end..]))
}

fn split_legacy_tail(tail: &str) -> Option<(&str, &str)> {
    let bytes = tail.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b' ' {
            index += 1;
            continue;
        }

        let run_start = index;
        while index < bytes.len() && bytes[index] == b' ' {
            index += 1;
        }

        if index - run_start < 2 || index < LEGACY_TERMINAL_WIDTH {
            continue;
        }

        let terminal = tail[..run_start].trim_end();
        let os = tail[index..].trim();

        if !terminal.is_empty() && !os.is_empty() {
            return Some((terminal, os));
        }
    }

    None
}

fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

fn validate_storage_field(field_name: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::message(format!(
            "{field_name} is unavailable, so the leaderboard cannot be updated"
        )));
    }

    if value.chars().any(char::is_control) {
        Err(AppError::message(format!(
            "{field_name} contains characters that cannot be saved to the leaderboard"
        )))
    } else {
        Ok(())
    }
}

fn pad_display_width(text: &str, width: usize) -> String {
    let mut padded = String::from(text);
    padded.push_str(&" ".repeat(width.saturating_sub(display_width(text))));
    padded
}

fn create_temporary_write_file(path: &Path) -> Result<(File, PathBuf), AppError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or("terminals.txt");
    let process_id = process::id();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut attempt = 0_u32;

    loop {
        let temp_path = parent.join(format!(".{file_name}.{process_id}.{unique}.{attempt}.tmp"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((file, temp_path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                attempt = attempt.saturating_add(1);
            }
            Err(error) => return Err(AppError::from(error)),
        }
    }
}

fn base_terminal_name(terminal: &str) -> String {
    let mut parts: Vec<&str> = terminal.split_whitespace().collect();

    while parts.last().is_some_and(|part| is_version_token(part)) {
        parts.pop();
    }

    parts.join(" ")
}

fn base_os_name(os: &str) -> String {
    let mut parts = Vec::new();

    for part in os.split_whitespace() {
        if starts_with_ascii_digit(part) {
            break;
        }

        parts.push(part);
    }

    parts.join(" ")
}

fn is_version_token(part: &str) -> bool {
    let stripped = part
        .strip_prefix('v')
        .or_else(|| part.strip_prefix('V'))
        .unwrap_or(part);

    starts_with_ascii_digit(stripped)
}

fn same_key_family(left: &str, right: &str) -> bool {
    left == right || is_missing_leading_character_variant(left, right)
}

fn is_missing_leading_character_variant(left: &str, right: &str) -> bool {
    match left.chars().count().cmp(&right.chars().count()) {
        Ordering::Equal => false,
        Ordering::Greater => without_first_character(left) == Some(right),
        Ordering::Less => without_first_character(right) == Some(left),
    }
}

fn without_first_character(text: &str) -> Option<&str> {
    let next_index = text.chars().next()?.len_utf8();
    Some(&text[next_index..])
}

fn starts_with_ascii_digit(part: &str) -> bool {
    part.chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
}

fn parse_number_groups(text: &str) -> Vec<u32> {
    let mut groups = Vec::new();
    let mut current = 0_u32;
    let mut in_group = false;

    for character in text.chars() {
        if let Some(digit) = character.to_digit(10) {
            current = current.saturating_mul(10).saturating_add(digit);
            in_group = true;
        } else if in_group {
            groups.push(current);
            current = 0;
            in_group = false;
        }
    }

    if in_group {
        groups.push(current);
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::{
        base_os_name, base_terminal_name, create_temporary_write_file, load_leaderboard,
        pad_display_width, validate_storage_field, AppError, Leaderboard, LeaderboardEntry,
        TerminalFeature, VersionKey,
    };

    #[test]
    fn feature_thresholds_are_stable() {
        assert_eq!(TerminalFeature::from_score(120), TerminalFeature::Goat);
        assert_eq!(TerminalFeature::from_score(200), TerminalFeature::Goat);
        assert_eq!(TerminalFeature::from_score(105), TerminalFeature::Sick);
        assert_eq!(TerminalFeature::from_score(90), TerminalFeature::Lit);
        assert_eq!(TerminalFeature::from_score(75), TerminalFeature::Mid);
        assert_eq!(TerminalFeature::from_score(60), TerminalFeature::Sus);
        assert_eq!(TerminalFeature::from_score(59), TerminalFeature::Trash);
    }

    #[test]
    fn feature_display_honors_alignment() {
        assert_eq!(format!("{:<5}", TerminalFeature::Goat), "goat ");
    }

    #[test]
    fn parses_legacy_feature_spacing_without_corruption() {
        let entry = LeaderboardEntry::parse(
            "01  120/120  goat  WezTerm 20240203-110809-5046fc22      macOS Tahoe 26.4.1 (25E253) arm64               ",
        )
        .unwrap();

        assert_eq!(
            entry.terminal,
            "WezTerm 20240203-110809-5046fc22".to_string()
        );
        assert_eq!(entry.os, "macOS Tahoe 26.4.1 (25E253) arm64".to_string());
    }

    #[test]
    fn parses_tab_separated_rows_with_internal_double_spaces() {
        let entry = LeaderboardEntry::parse(
            "01\t120/120\tgoat\tApple  Terminal 470\tmacOS  Tahoe 26.4.1 (25E253) arm64",
        )
        .unwrap();

        assert_eq!(entry.terminal, "Apple  Terminal 470".to_string());
        assert_eq!(entry.os, "macOS  Tahoe 26.4.1 (25E253) arm64".to_string());
    }

    #[test]
    fn preserves_whitespace_in_tab_separated_terminal_and_os_fields() {
        let entry =
            LeaderboardEntry::parse("01\t120/120\tgoat\t WezTerm \t macOS Tahoe 26.4.1 ").unwrap();

        assert_eq!(entry.terminal, " WezTerm ".to_string());
        assert_eq!(entry.os, " macOS Tahoe 26.4.1 ".to_string());
    }

    #[test]
    fn rejects_rows_with_empty_os_field() {
        assert!(LeaderboardEntry::parse("01\t120/120\tgoat\tWezTerm\t").is_none());
    }

    #[test]
    fn parses_legacy_rows_with_long_terminal_names() {
        let entry = LeaderboardEntry::parse(
            "01  120/120  goat  Very Long Terminal Name Beyond Thirty Six 1.2.3  macOS Tahoe 26.4.1 (25E253) arm64",
        )
        .unwrap();

        assert_eq!(
            entry.terminal,
            "Very Long Terminal Name Beyond Thirty Six 1.2.3".to_string()
        );
        assert_eq!(entry.os, "macOS Tahoe 26.4.1 (25E253) arm64".to_string());
    }

    #[test]
    fn parses_legacy_rows_without_separator_after_terminal_width() {
        let entry = LeaderboardEntry::parse(
            "01  120/120  goat  Apple Terminal 470                   macOS Tahoe 26.4.1 (25E253) arm64",
        )
        .unwrap();

        assert_eq!(entry.terminal, "Apple Terminal 470".to_string());
        assert_eq!(entry.os, "macOS Tahoe 26.4.1 (25E253) arm64".to_string());
    }

    #[test]
    fn write_and_load_round_trip_preserves_fields() {
        use std::env;
        use std::fs;
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "termfactor-leaderboard-{}-{unique}.txt",
            process::id()
        ));

        let result = (|| -> Result<(), AppError> {
            let mut leaderboard = Leaderboard::default();
            leaderboard.record(LeaderboardEntry::new(
                120,
                TerminalFeature::Goat,
                " WezTerm  20240203 ",
                " macOS  Tahoe 26.4.1 ",
            ));
            leaderboard.write(&path)?;

            let loaded = Leaderboard::load_existing(&path)?.unwrap();
            assert_eq!(loaded.entries.len(), 1);
            assert_eq!(
                loaded.entries[0].terminal,
                " WezTerm  20240203 ".to_string()
            );
            assert_eq!(loaded.entries[0].os, " macOS  Tahoe 26.4.1 ".to_string());

            Ok(())
        })();

        let _ = fs::remove_file(&path);
        result.unwrap();
    }

    #[test]
    fn pads_terminal_names_by_display_width() {
        assert_eq!(pad_display_width("猫", 4), "猫  ".to_string());
    }

    #[test]
    fn loads_legacy_file_when_preferred_path_is_missing() {
        use std::env;
        use std::fs;
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            env::temp_dir().join(format!("termfactor-legacy-load-{}-{unique}", process::id()));
        let preferred_path = directory.join("preferred.txt");
        let legacy_path = directory.join("legacy.txt");

        let result = (|| -> Result<(), AppError> {
            fs::create_dir_all(&directory)?;
            fs::write(
                &legacy_path,
                "# term factors\n\n01\t120/120\tgoat\tWezTerm 20240203\tmacOS Tahoe 26.4.1\n",
            )?;

            let leaderboard = load_leaderboard(&preferred_path, &legacy_path)?;
            assert_eq!(leaderboard.entries.len(), 1);
            assert_eq!(
                leaderboard.entries[0].terminal,
                "WezTerm 20240203".to_string()
            );

            Ok(())
        })();

        let _ = fs::remove_file(&legacy_path);
        let _ = fs::remove_dir(&directory);
        result.unwrap();
    }

    #[test]
    fn loads_legacy_file_when_preferred_path_has_no_entries() {
        use std::env;
        use std::fs;
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "termfactor-empty-preferred-load-{}-{unique}",
            process::id()
        ));
        let preferred_path = directory.join("preferred.txt");
        let legacy_path = directory.join("legacy.txt");

        let result = (|| -> Result<(), AppError> {
            fs::create_dir_all(&directory)?;
            fs::write(&preferred_path, "# term factors\n\n")?;
            fs::write(
                &legacy_path,
                "# term factors\n\n01\t120/120\tgoat\tWezTerm 20240203\tmacOS Tahoe 26.4.1\n",
            )?;

            let leaderboard = load_leaderboard(&preferred_path, &legacy_path)?;
            assert_eq!(leaderboard.entries.len(), 1);
            assert_eq!(
                leaderboard.entries[0].terminal,
                "WezTerm 20240203".to_string()
            );

            Ok(())
        })();

        let _ = fs::remove_file(&preferred_path);
        let _ = fs::remove_file(&legacy_path);
        let _ = fs::remove_dir(&directory);
        result.unwrap();
    }

    #[test]
    fn rejects_tab_separated_rows_with_extra_fields() {
        assert!(
            LeaderboardEntry::parse("01\t120/120\tgoat\tWezTerm\tmacOS Tahoe 26.4.1\textra")
                .is_none()
        );
    }

    #[test]
    fn accepts_tab_separated_rows_with_trailing_tab() {
        let entry =
            LeaderboardEntry::parse("01\t120/120\tgoat\tWezTerm\tmacOS Tahoe 26.4.1\t").unwrap();

        assert_eq!(entry.terminal, "WezTerm".to_string());
        assert_eq!(entry.os, "macOS Tahoe 26.4.1".to_string());
    }

    #[test]
    fn rejects_storage_fields_with_tabs() {
        assert!(validate_storage_field("terminal", "Wez\tTerm").is_err());
        assert!(validate_storage_field("os", "macOS\tTahoe").is_err());
    }

    #[test]
    fn rejects_storage_fields_with_newlines() {
        assert!(validate_storage_field("terminal", "WezTerm\n470").is_err());
        assert!(validate_storage_field("os", "macOS\rTahoe").is_err());
    }

    #[test]
    fn rejects_storage_fields_with_escape_sequences() {
        assert!(validate_storage_field("terminal", "WezTerm\x1b[31m").is_err());
        assert!(validate_storage_field("os", "macOS\x07Tahoe").is_err());
    }

    #[test]
    fn rejects_empty_storage_fields() {
        assert!(validate_storage_field("terminal", "").is_err());
        assert!(validate_storage_field("os", "   ").is_err());
    }

    #[test]
    fn write_fails_when_terminal_contains_storage_separator() {
        use std::env;
        use std::fs;
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "termfactor-invalid-write-{}-{unique}.txt",
            process::id()
        ));

        let result: Result<(), AppError> = {
            let mut leaderboard = Leaderboard::default();
            leaderboard.record(LeaderboardEntry::new(
                120,
                TerminalFeature::Goat,
                "Wez\tTerm",
                "macOS Tahoe 26.4.1",
            ));
            leaderboard.write(&path)
        };

        let _ = fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn temporary_write_files_use_unique_names() {
        use std::env;
        use std::fs;
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            env::temp_dir().join(format!("termfactor-temp-write-{}-{unique}", process::id()));
        let path = directory.join("terminals.txt");

        let result = (|| -> Result<(), AppError> {
            fs::create_dir_all(&directory)?;
            let (first_file, first_path) = create_temporary_write_file(&path)?;
            let (second_file, second_path) = create_temporary_write_file(&path)?;

            drop(first_file);
            drop(second_file);

            assert_ne!(first_path, second_path);

            fs::remove_file(first_path)?;
            fs::remove_file(second_path)?;

            Ok(())
        })();

        let _ = fs::remove_dir(&directory);
        result.unwrap();
    }

    #[test]
    fn trims_versions_from_terminal_family() {
        assert_eq!(
            base_terminal_name("WezTerm 20240203-110809-5046fc22"),
            "WezTerm".to_string()
        );
        assert_eq!(base_terminal_name("Ghostty v1.0.0"), "Ghostty".to_string());
    }

    #[test]
    fn trims_version_from_os_family() {
        assert_eq!(base_os_name("Fedora Linux 40"), "Fedora Linux".to_string());
    }

    #[test]
    fn does_not_match_terminal_families_with_multiple_missing_leading_characters() {
        assert!(!super::same_key_family("WezTerm", "zzWezTerm"));
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(VersionKey::from_os("macOS 14.10") > VersionKey::from_os("macOS 14.9"));
    }

    #[test]
    fn prefers_higher_score_before_newer_os_version() {
        let mut leaderboard = Leaderboard::default();
        leaderboard.record(LeaderboardEntry::new(
            120,
            TerminalFeature::Goat,
            "WezTerm 20240203",
            "macOS 14.4",
        ));
        leaderboard.record(LeaderboardEntry::new(
            95,
            TerminalFeature::Lit,
            "WezTerm 20240203",
            "macOS 14.5",
        ));

        assert_eq!(leaderboard.entries.len(), 1);
        assert_eq!(leaderboard.entries[0].score, 120);
        assert_eq!(leaderboard.entries[0].os, "macOS 14.4".to_string());
    }

    #[test]
    fn prefers_newer_os_version_when_score_is_tied() {
        let mut leaderboard = Leaderboard::default();
        leaderboard.record(LeaderboardEntry::new(
            105,
            TerminalFeature::Sick,
            "WezTerm 20240203",
            "macOS 14.4",
        ));
        leaderboard.record(LeaderboardEntry::new(
            105,
            TerminalFeature::Sick,
            "WezTerm 20240203",
            "macOS 14.5",
        ));

        assert_eq!(leaderboard.entries.len(), 1);
        assert_eq!(leaderboard.entries[0].score, 105);
        assert_eq!(leaderboard.entries[0].os, "macOS 14.5".to_string());
    }

    #[test]
    fn prefers_higher_score_for_same_os_version() {
        let mut leaderboard = Leaderboard::default();
        leaderboard.record(LeaderboardEntry::new(
            80,
            TerminalFeature::Mid,
            "WezTerm 20240203",
            "Fedora Linux 40",
        ));
        leaderboard.record(LeaderboardEntry::new(
            105,
            TerminalFeature::Sick,
            "WezTerm 20240203",
            "Fedora Linux 40",
        ));

        assert_eq!(leaderboard.entries.len(), 1);
        assert_eq!(leaderboard.entries[0].score, 105);
        assert_eq!(leaderboard.entries[0].feature, TerminalFeature::Sick);
    }

    #[test]
    fn repairs_one_character_truncation_duplicates() {
        let mut leaderboard = Leaderboard::default();
        leaderboard.record(LeaderboardEntry::new(
            120,
            TerminalFeature::Goat,
            "ezTerm 20240203-110809-5046fc22",
            "acOS Tahoe 26.4.1 (25E253) arm64",
        ));
        leaderboard.record(LeaderboardEntry::new(
            120,
            TerminalFeature::Goat,
            "WezTerm 20240203-110809-5046fc22",
            "macOS Tahoe 26.4.1 (25E253) arm64",
        ));

        assert_eq!(leaderboard.entries.len(), 1);
        assert_eq!(
            leaderboard.entries[0].terminal,
            "WezTerm 20240203-110809-5046fc22".to_string()
        );
        assert_eq!(
            leaderboard.entries[0].os,
            "macOS Tahoe 26.4.1 (25E253) arm64".to_string()
        );
    }

    #[test]
    fn console_rows_do_not_end_with_padding() {
        let mut entry = LeaderboardEntry::new(
            65,
            TerminalFeature::Sus,
            "Apple Terminal 470",
            "macOS Tahoe 26.4.1 (25E253) arm64",
        );
        entry.rank = 2;

        let row = entry.console_row(super::display_width(&entry.terminal));

        assert!(!row.ends_with(' '));
    }
}
