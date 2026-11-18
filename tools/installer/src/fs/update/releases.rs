use super::{REPO, Route, fetch};
use anyhow::{Result, bail};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub(crate) struct Offer {
    pub version: Version,
    pub route: Route,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

impl Release {
    fn candidate(&self, current: &Version) -> Option<Version> {
        let version = Version::parse(self.tag_name.strip_prefix('v')?).ok()?;
        let is_prerelease = !version.pre.is_empty();
        let channel_accepts = !is_prerelease || !current.pre.is_empty();
        (!self.draft
            && self.prerelease == is_prerelease
            && channel_accepts
            && version.cmp_precedence(current).is_gt())
        .then_some(version)
    }
}

pub(super) fn newest(client: &Client, current: &Version) -> Result<Option<Version>> {
    let api = format!("https://api.github.com/repos/{REPO}/releases");
    if current.pre.is_empty() {
        let release: Release =
            serde_json::from_slice(&fetch(client, &format!("{api}/latest"), 1 << 20)?)?;
        return Ok(release.candidate(current));
    }
    let mut best = None;
    for page in 1..=20 {
        let url = format!("{api}?per_page=100&page={page}");
        let items: Vec<Release> = serde_json::from_slice(&fetch(client, &url, 16 << 20)?)?;
        best = items
            .iter()
            .filter_map(|r| r.candidate(current))
            .chain(best)
            .max_by(Version::cmp_precedence);
        if items.len() < 100 {
            return Ok(best);
        }
    }
    bail!("Release history exceeds the lookup limit")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn channels_never_downgrade_or_cross_from_stable_to_beta() {
        for (current, tag, draft, prerelease, eligible) in [
            ("2.0.0-beta", "v2.0.0-beta.1", false, true, true),
            ("2.0.0-beta.1", "v2.0.0", false, false, true),
            ("2.0.0-beta.10", "v2.0.0-beta.2", false, true, false),
            ("2.0.0", "v2.1.0-beta", false, true, false),
            ("2.0.0-beta", "v2.0.0", true, false, false),
            ("2.0.0", "v2.0.0+rebuild", false, false, false),
            ("2.0.0", "v3.0.0-beta", false, false, false),
        ] {
            let item = Release {
                tag_name: tag.into(),
                draft,
                prerelease,
            };
            assert_eq!(
                item.candidate(&Version::parse(current).unwrap()).is_some(),
                eligible
            );
        }
    }
}
