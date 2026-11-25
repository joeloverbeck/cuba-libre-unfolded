//! Where a faction's sessions run (#205): one small folder for each tool and the client
//! program, outside the checkout, in the data directory. The shared library writes them
//! (`unfolded_agent_players::folder`, which says what was measured of them). The places,
//! the instructions and what Claude Code may not read are Cuba Libre's. `cuba-tools
//! player-folders` writes them, and the line the human pastes names them.

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
use unfolded_agent_players::folder::{self, CODEX_WAIT, Folders};

/// The client program's name, as the instructions in the checkout write it.
pub const CLIENT: &str = "cuba-player";

/// The places in a data directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Places {
    data: PathBuf,
}

impl Places {
    pub fn of(data: &Path) -> Self {
        Self {
            data: data.to_owned(),
        }
    }

    /// In this computer's data directory: `cuba-libre-unfolded` in `$XDG_DATA_HOME`, or
    /// in `~/.local/share`.
    pub fn here() -> Option<Self> {
        let set = |name| std::env::var_os(name).map(PathBuf::from);
        set("XDG_DATA_HOME")
            .filter(|path| path.is_absolute())
            .or_else(|| set("HOME").map(|home| home.join(".local/share")))
            .filter(|path| path.is_absolute())
            .map(|data| Self::of(&data.join("cuba-libre-unfolded")))
    }

    /// The directory of the two tools' folders.
    pub fn folders(&self) -> PathBuf {
        self.data.join("player")
    }

    /// That directory for the pasted line, whose sheet holds three lines: `~` is the home.
    pub fn folders_in_short(&self) -> String {
        let folders = self.folders();
        std::env::var_os("HOME")
            .and_then(|home| Some(format!("~/{}", folders.strip_prefix(home).ok()?.display())))
            .unwrap_or_else(|| folders.display().to_string())
    }

    pub fn claude_code(&self) -> PathBuf {
        self.folders().join("claude-code")
    }

    pub fn codex(&self) -> PathBuf {
        self.folders().join("codex")
    }

    pub fn client(&self) -> PathBuf {
        self.data.join("client").join(CLIENT)
    }

    /// The player instructions, with the client's path in place of its name and the shared
    /// library's words for how Codex waits.
    pub fn instructions(&self) -> String {
        include_str!("../docs/player-instructions.md")
            .replace(CLIENT, &self.client().display().to_string())
            .replace("{CODEX_WAIT}", CODEX_WAIT)
    }

    /// Write both folders, deny Claude Code's file tools the saves and the checkout, and
    /// install the `built` client. A folder that holds anything else is refused first.
    pub fn write(&self, built: &Path) -> Result<()> {
        let unread = [
            unreadable(&self.data.join("saves"))?,
            unreadable(&checkout())?,
        ];
        Folders {
            claude_code: &self.claude_code(),
            codex: &self.codex(),
            client: &self.client(),
            instructions: &self.instructions(),
        }
        .write()
        .context("cannot write the player's folders")?;
        self.deny_reading(&unread)
            .context("cannot deny Claude Code the saves and the checkout")?;
        folder::install(built, &self.client()).context("cannot install the player's client")
    }

    /// Add to the settings the library wrote: the save holds the match's seed. Codex has
    /// no such denial, since one would take its client's permission away (#205).
    fn deny_reading(&self, unread: &[String]) -> Result<()> {
        let path = self.claude_code().join(".claude/settings.json");
        let mut settings: Value = serde_json::from_slice(&fs::read(&path)?)?;
        settings
            .pointer_mut("/permissions/deny")
            .and_then(Value::as_array_mut)
            .context("the folder's settings list no denials")?
            .extend(unread.iter().map(|rule| json!(rule)));
        fs::write(&path, format!("{settings:#}\n"))?;
        Ok(())
    }
}

/// Claude Code's rule for all that lies in `directory`. Two slashes begin a path that is not
/// the folder's own. A pattern reads more than these characters as something else.
fn unreadable(directory: &Path) -> Result<String> {
    let plain = |c: char| c.is_ascii_alphanumeric() || "/._-".contains(c);
    match directory.to_str() {
        Some(path) if path.chars().all(plain) => Ok(format!("Read(/{path}/**)")),
        _ => bail!(
            "{} must be a path of ASCII letters, digits, \"/\", \".\", \"_\" and \"-\" only, so that a setting can deny it",
            directory.display()
        ),
    }
}

/// The checkout this was built from or, for a worktree inside one, the checkout that holds
/// it: saves and captures of either lie there.
pub fn checkout() -> PathBuf {
    let built = Path::new(env!("CARGO_MANIFEST_DIR"));
    built
        .ancestors()
        .find(|directory| directory.join(".git").is_dir())
        .unwrap_or(built)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_that_a_rule_would_not_name_is_refused() {
        assert_eq!(
            unreadable(Path::new("/home/a-b/src/cuba_1.0")).unwrap(),
            "Read(//home/a-b/src/cuba_1.0/**)"
        );
        for wrong in ["/home/a b", "/home/*", "/home/[a]", "/home/a)", "/home/ñ"] {
            assert!(unreadable(Path::new(wrong)).is_err(), "{wrong}");
        }
    }
}
