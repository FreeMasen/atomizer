use std::path::PathBuf;

use atom_syndication::Feed;
use url::Url;

mod config;
mod error;
mod feed;

pub use config::{Config, FeedConfig};
pub use error::Error;
pub use feed::Feeds;

pub(crate) type Result<T = (), E = Error> = std::result::Result<T, E>;

pub async fn fetch_feed(url: Url) -> Result<Feed> {
    match url.scheme() {
        "http" | "https" => fetch_feed_reqwest(url).await,
        "file" => fetch_feed_fs(url).await,
        scheme => {
            log::warn!("bad scheme in url: {url}");
            return Err(Error::UnknownScheme(scheme.to_string()));
        }
    }
}

async fn fetch_feed_reqwest(url: Url) -> Result<Feed> {
    let raw_feed = reqwest::get(url.clone())
        .await
        .inspect_err(|e| log::warn!("Error fetching {url}: {e}"))?
        .error_for_status()
        .inspect_err(|e| {
            log::warn!("Error fetching url: {e}");
        })?
        .text()
        .await
        .inspect_err(|e| log::warn!("Fetching feed failed to read as text: {e}"))?;
    Ok(
        atom_syndication::Feed::read_from(raw_feed.as_bytes()).inspect_err(|e| {
            log::warn!("Failed to deserialize feed: {e}\n`{raw_feed}`");
        })?,
    )
}

async fn fetch_feed_fs(url: Url) -> Result<Feed> {
    let path = url.to_file_path().map_err(|_| {
        log::warn!("cannot convert url to file path: {url}");
        Error::InvalidFileUrl(url)
    })?;
    let raw_feed = tokio::fs::read_to_string(&path)
        .await
        .inspect_err(|e| log::warn!("Error reading path `{}`: {e}", path.display()))?;
    Ok(
        atom_syndication::Feed::read_from(raw_feed.as_bytes()).inspect_err(|e| {
            log::warn!("Failed to deserialize feed: {e}\n`{raw_feed}`");
        })?,
    )
}

pub async fn get_config() -> Result<Config> {
    let path = get_config_path(None);
    let toml_str = tokio::fs::read_to_string(&path).await.inspect_err(|e| {
        log::warn!("failed to read config at path `{}`: {e}", path.display());
    })?;
    Ok(toml::from_str(&toml_str).inspect_err(|e| {
        log::warn!("Bad toml in config: {e} \n`{toml_str}`");
    })?)
}

pub async fn save_config(config: &Config, path: impl Into<Option<PathBuf>>) -> Result {
    let path = path.into().unwrap_or_else(|| get_config_path(None));
    if !path.exists() {
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    }
    let toml_str = toml::to_string_pretty(config)
        .inspect_err(|e| log::warn!("Error serializing toml: {e}\n{config:#?}"))?;
    tokio::fs::write(&path, toml_str).await.inspect_err(|e| {
        log::warn!("Error writing toml to `{}`: {e}", path.display());
    })?;
    Ok(())
}

pub async fn get_feeds() -> Result<Feeds> {
    let path = get_feeds_path(None);
    let toml_str = tokio::fs::read_to_string(&path).await.inspect_err(|e| {
        log::warn!("failed to read feeds at path `{}`: {e}", path.display());
    })?;
    Ok(toml::from_str(&toml_str).inspect_err(|e| {
        log::warn!("Bad toml in feeds: {e} \n`{toml_str}`");
    })?)
}

pub async fn save_feeds(feeds: &Feeds, path: impl Into<Option<PathBuf>>) -> Result {
    let path = path.into().unwrap_or_else(|| get_feeds_path(None));
    if !path.exists() {
        tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    }
    let toml_str = toml::to_string_pretty(feeds)
        .inspect_err(|e| log::warn!("Error serializing toml: {e}\n{feeds:#?}"))?;
    tokio::fs::write(&path, toml_str).await.inspect_err(|e| {
        log::warn!("Error writing toml to `{}`: {e}", path.display());
    })?;
    Ok(())
}

pub fn get_config_path(base_path: impl Into<Option<PathBuf>>) -> PathBuf {
    base_path
        .into()
        .unwrap_or_else(get_project_dir)
        .join("config.toml")
}

pub fn get_feeds_path(base_path: impl Into<Option<PathBuf>>) -> PathBuf {
    base_path
        .into()
        .unwrap_or_else(get_project_dir)
        .join("feeds.toml")
}

pub fn get_project_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "freemasen", "atomizer")
        .unwrap()
        .config_dir()
        .to_path_buf()
}
