use std::str::FromStr;

use tower_lsp::lsp_types::*;

use crate::util::LspUriExt;

#[derive(Debug, Clone, Copy)]
pub enum ToolName {
    Cargo,
    Rokit,
    Wally,
}

impl ToolName {
    pub fn from_uri(uri: &Url) -> Result<Self, &'static str> {
        match uri.file_name() {
            Some(file_name) => file_name.parse(),
            None => Err("No file name"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Cargo, Self::Rokit, Self::Wally]
    }

    pub fn file_glob(&self) -> &'static str {
        match self {
            Self::Cargo => "**/Cargo.{toml,lock}",
            Self::Rokit => "**/rokit.toml",
            Self::Wally => "**/wally.{toml,lock}",
        }
    }

    pub fn relevant_file_uris(&self, uri: &Url) -> Vec<Url> {
        match self {
            Self::Cargo => match uri.file_name().as_deref() {
                Some("Cargo.lock") => vec![uri.with_file_name("Cargo.toml").unwrap()],
                Some("Cargo.toml") => {
                    let mut lockfiles = Vec::new();
                    let mut current_dir = uri.to_file_path().unwrap();
                    loop {
                        current_dir.pop();
                        let lockfile = current_dir.join("Cargo.lock");
                        if lockfile.exists() {
                            lockfiles.push(Url::from_file_path(lockfile).unwrap());
                        }
                        if !current_dir.pop() {
                            break;
                        }
                    }
                    lockfiles
                }
                _ => Vec::new(),
            },
            Self::Rokit => Vec::new(),
            Self::Wally => match uri.file_name().as_deref() {
                Some("wally.toml") => vec![uri.with_file_name("wally.lock").unwrap()],
                Some("wally.lock") => vec![uri.with_file_name("wally.toml").unwrap()],
                _ => Vec::new(),
            },
        }
    }
}

impl FromStr for ToolName {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "cargo" | "cargo.toml" | "cargo.lock" => Ok(Self::Cargo),
            "rokit" | "rokit.toml" => Ok(Self::Rokit),
            "wally" | "wally.toml" | "wally.lock" => Ok(Self::Wally),
            _ => Err("Unknown tool"),
        }
    }
}
