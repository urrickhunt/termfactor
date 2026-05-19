use std::env;
use std::path::PathBuf;

pub fn leaderboard_path() -> PathBuf {
    data_directory()
        .unwrap_or_else(fallback_directory)
        .join("termfactor")
        .join("terminals.txt")
}

pub fn legacy_leaderboard_path() -> PathBuf {
    fallback_directory().join("terminals.txt")
}

fn data_directory() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        home_directory().map(|path| path.join("Library").join("Application Support"))
    }

    #[cfg(windows)]
    {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home_directory().map(|path| path.join(".local").join("share")))
    }
}

fn fallback_directory() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(not(windows))]
fn home_directory() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
