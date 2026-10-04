//! The installer's only call into the OS for the home directory
//! (`docs/ARCHITECTURE.md` `arch-home-dir-single-source`), so a test or a
//! future override has one place to change.

use std::path::PathBuf;

use anyhow::{Result, anyhow};

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn require_home_dir() -> Result<PathBuf> {
    home_dir().ok_or_else(|| anyhow!("Cannot find home directory"))
}
