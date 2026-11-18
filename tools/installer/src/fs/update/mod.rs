//! Updates the installer and its bundled sources as one release.
mod archive;
mod bundle;
mod origin;
mod releases;
#[cfg(windows)]
mod windows;
pub(crate) use origin::Route;
pub(crate) use releases::Offer;

use anyhow::{Context, Result, ensure};
use reqwest::blocking::Client;
use semver::Version;
use std::{io::Read, time::Duration};

const REPO: &str = "devsepnine/hibi_ai";
const PUBLIC_KEY: &str = include_str!("../../../assets/update.pub");

fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!("hibi/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(180))
        .build()?)
}

fn fetch(client: &Client, url: &str, limit: u64) -> Result<Vec<u8>> {
    let mut response = client.get(url).send()?.error_for_status()?;
    ensure!(
        response.content_length().unwrap_or(0) <= limit,
        "Download exceeds size limit"
    );
    let mut bytes = Vec::new();
    (&mut response).take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "Download exceeds size limit");
    Ok(bytes)
}

pub(crate) fn check() -> Result<Option<Offer>> {
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let client = client()?;
    let Some(version) = releases::newest(&client, &current)? else {
        return Ok(None);
    };
    let route = origin::resolve(&std::env::current_exe()?.canonicalize()?, &version, &client)?;
    Ok(Some(Offer { version, route }))
}

pub(crate) fn apply(offer: Offer) -> Result<()> {
    let client = client()?;
    let exe = std::env::current_exe()?.canonicalize()?;
    let route = origin::resolve(&exe, &offer.version, &client)?;
    ensure!(
        route == offer.route,
        "Install location or package version changed. Check again."
    );
    match route {
        Route::Direct(root) => bundle::install(&client, &root, &offer.version),
        Route::Brew { program } => origin::upgrade_brew(&program, &offer.version),
        #[cfg(windows)]
        Route::Scoop => windows::upgrade_scoop(&exe, &offer.version),
        Route::Pending | Route::Manual => {
            anyhow::bail!("Use the official release package for this installation")
        }
    }
}

pub(crate) fn open_release(version: &Version) -> Result<()> {
    let url = format!("https://github.com/{REPO}/releases/tag/v{version}");
    #[cfg(windows)]
    let program = "explorer.exe";
    #[cfg(target_os = "macos")]
    let program = "/usr/bin/open";
    #[cfg(not(any(windows, target_os = "macos")))]
    let program = "xdg-open";
    crate::exec::launch(std::process::Command::new(program).arg(url)).context("Open release page")
}

fn platform() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_env = "musl") {
        "linux-musl"
    } else {
        "linux"
    }
}

fn executable() -> &'static str {
    if cfg!(windows) { "hibi.exe" } else { "hibi" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_and_oversized_responses_are_errors() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };
        for (response, limit) in [
            ("HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\nshort", 40),
            ("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\noversize", 4),
            ("HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\n\r\n", 32),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let worker = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 2048];
                assert!(stream.read(&mut request).unwrap() > 0);
                stream.write_all(response.as_bytes()).unwrap();
            });
            let client = Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap();
            assert!(fetch(&client, &url, limit).is_err());
            worker.join().unwrap();
        }
    }

    #[test]
    #[ignore = "queries live GitHub release and package metadata"]
    fn live_release_lookup() -> Result<()> {
        let client = client()?;
        let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
        let _ = check()?;
        let stable =
            releases::newest(&client, &Version::new(0, 0, 0))?.context("Missing stable release")?;
        assert!(stable.pre.is_empty());
        if let Some(exe) = std::env::var_os("HIBI_TEST_BREW_BIN") {
            let root = std::path::Path::new(&exe).canonicalize()?;
            assert!(matches!(
                origin::resolve(&root, &stable, &client)?,
                Route::Brew { .. }
            ));
            assert!(matches!(
                origin::resolve(&root, &current, &client)?,
                Route::Pending
            ));
        }
        Ok(())
    }
}
