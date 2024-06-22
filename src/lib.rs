use std::{collections::HashMap, path::PathBuf};

use atom_syndication::Feed;
use config::Config;
use feed::Feeds;
use pager_rs::{CommandList, State, StatusBar};
use url::Url;

mod config;
mod feed;

pub(crate) type Result<T = (), E = Error> = std::result::Result<T, E>;

pub async fn run_update() -> Result {
    let mut config = get_config().await?;
    let mut feeds = get_feeds().await?;
    for feed in config.feeds.iter_mut() {
        let feed2 = fetch_feed(feed.url.clone()).await?;
        if let Some(existing_feed) = feeds.feeds.iter_mut().find(|f| f.id == feed.id) {
            feed.last_updated = feed2.updated;
            feed.read
                .retain(|id| feed2.entries().iter().any(|e| &e.id == id));
            *existing_feed = feed2;
        } else {
            feeds.feeds.push(feed2);
        }
    }
    save_config(&config, None).await?;
    save_feeds(&feeds, None).await?;
    Ok(())
}

pub async fn run_read(category_filter: Option<String>, all: bool) -> Result {
    let mut config = get_config().await?;
    if config.feeds.is_empty() {
        println!("No feeds to read");
        std::process::exit(1);
    }
    'feeds: loop {
        let idx = dialoguer::Select::new()
            .items(&config.feeds.iter().map(|f| &f.name).collect::<Vec<_>>())
            .default(0)
            .interact_opt()?;
        let Some(feed_idx) = idx else {
            break 'feeds;
        };
        let feeds = get_feeds().await?;
        if let Some(feed_cfg) = config.feeds.get_mut(feed_idx) {
            let Some(feed) = feeds.feeds.iter().find(|f| f.id == feed_cfg.id) else {
                break 'feeds;
            };
            'entries: loop {
                let unread_entries: Vec<_> = feed
                    .entries
                    .iter()
                    .filter(|e| {
                        if let Some(cat) = category_filter.as_ref() {
                            if !e.categories.iter().any(|c| c.term.starts_with(cat)) {
                                return false;
                            }
                        }
                        all || !feed_cfg.read.contains(&e.id)
                    })
                    .collect();
                if unread_entries.is_empty() {
                    println!("No unread entries");
                    break 'entries;
                }
                let unread_titles: Vec<String> =
                    unread_entries.iter().map(|e| e.title.to_string()).collect();
                let selected_entry = dialoguer::Select::new()
                    .with_prompt("Unread")
                    .items(&unread_titles)
                    .default(0)
                    .max_length(10)
                    .interact_opt()?;
                let Some(selected_entry) = selected_entry else {
                    break 'entries;
                };
                let entry = unread_entries
                    .get(selected_entry)
                    .expect("entry index in range");
                let Some(ct) = entry.content() else {
                    break 'entries;
                };
                let ct_type = ct
                    .content_type
                    .as_ref()
                    .map(|c| c.as_str())
                    .unwrap_or("text");
                let ct_value = ct.value.as_ref().map(|v| v.as_str()).unwrap_or("");
                let content = if ct_type.ends_with("html") {
                    htmd::convert(ct_value).unwrap()
                } else {
                    ct_value.to_string()
                };

                let status_bar = StatusBar::new(entry.title.to_string());

                let mut state = State::new(content, status_bar, CommandList::default())?;
                state.show_line_numbers = false;
                state.word_wrap = true;
                pager_rs::init()?;
                pager_rs::run(&mut state)?;
                pager_rs::finish()?;
                feed_cfg.read.push(entry.id.clone());
            }
        } else {
            eprintln!("feed idx not found: {feed_idx} len: {config:#?}");
        }
    }
    save_config(&config, None).await
}

pub async fn run_categories() -> Result {
    let feeds = get_feeds().await?;
    let mut categories: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for feed in feeds.feeds.iter() {
        for entry in feed.entries() {
            for cat in entry.categories() {
                categories.entry(cat.term.to_string()).and_modify(|v| {
                    v.entry(feed.title.to_string()).and_modify(|f: &mut usize| {
                        *f += 1;
                    }).or_insert_with(|| 1);
                }).or_insert_with(|| {
                    HashMap::from_iter([(feed.title.to_string(), 1)])
                });
            }
        }
    }
    for (cat, feeds) in categories {
        println!("{cat}:");
        for (feed, ct) in feeds {
            println!("  {feed}: {ct}");
        }
        println!("----------");
    }
    Ok(())
}

pub async fn run_setup(force: bool) -> Result {
    let proj_dir = get_project_dir();
    let config_path = get_config_path(proj_dir.clone());
    let feeds_path = get_feeds_path(proj_dir);
    if !force && (config_path.exists() || feeds_path.exists()) {
        log::error!("Previously setup, use --force (-f) to overwrite exiting config");
        std::process::exit(1);
    }
    let default_config = Config::default();
    let default_feeds = Feeds::default();
    save_config(&default_config, config_path).await?;
    save_feeds(&default_feeds, feeds_path).await?;
    Ok(())
}

pub async fn run_add(url: Url) -> Result {
    let mut config = get_config().await?;
    let mut feeds = get_feeds().await?;
    let feed = fetch_feed(url.clone()).await?;
    config.feeds.push(config::FeedConfig {
        id: feed.id.clone(),
        name: feed.title.to_string(),
        url,
        last_updated: feed.updated,
        read: Vec::new(),
    });
    feeds.feeds.push(feed);
    save_config(&config, None).await?;
    save_feeds(&feeds, None).await?;
    Ok(())
}

pub async fn run_remove(id_or_name: String) -> Result {
    let mut config = get_config().await?;
    let mut feeds = get_feeds().await?;
    let Some(feed_idx) = config.feeds.iter().enumerate().find_map(|(idx, f)| {
        if f.id == id_or_name || f.name == id_or_name {
            return Some(idx);
        }
        None
    }) else {
        eprintln!("Unknown feed `{id_or_name}`");
        std::process::exit(1);
    };
    let feed = config.feeds.remove(feed_idx);
    feeds.feeds.retain(|f| f.id != feed.id);
    save_config(&config, None).await?;
    save_feeds(&feeds, None).await?;
    Ok(())
}

pub async fn run_config(delete: bool, key: Option<String>, value: Option<String>) -> Result {
    let mut config = get_config().await?;
    match (key, value) {
        (Some(key), Some(value)) => {
            config.update_key(&key, value)?;
        }
        (Some(key), None) => {
            if delete {
                config.delete_key(&key)?;
            } else {
                config.report_key(&key)?;
            }
        }
        (None, Some(_)) => {
            eprintln!("Invalid arguments, no key for value");
            std::process::exit(1);
        }
        (None, None) => {
            println!("{config}");
            return Ok(());
        }
    }
    save_config(&config, None).await
}

async fn fetch_feed(url: Url) -> Result<Feed> {
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

async fn get_config() -> Result<Config> {
    let path = get_config_path(None);
    let toml_str = tokio::fs::read_to_string(&path).await.inspect_err(|e| {
        log::warn!("failed to read config at path `{}`: {e}", path.display());
    })?;
    Ok(toml::from_str(&toml_str).inspect_err(|e| {
        log::warn!("Bad toml in config: {e} \n`{toml_str}`");
    })?)
}

async fn save_config(config: &Config, path: impl Into<Option<PathBuf>>) -> Result {
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

async fn get_feeds() -> Result<Feeds> {
    let path = get_feeds_path(None);
    let toml_str = tokio::fs::read_to_string(&path).await.inspect_err(|e| {
        log::warn!("failed to read feeds at path `{}`: {e}", path.display());
    })?;
    Ok(toml::from_str(&toml_str).inspect_err(|e| {
        log::warn!("Bad toml in feeds: {e} \n`{toml_str}`");
    })?)
}

async fn save_feeds(feeds: &Feeds, path: impl Into<Option<PathBuf>>) -> Result {
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

fn get_config_path(base_path: impl Into<Option<PathBuf>>) -> PathBuf {
    base_path
        .into()
        .unwrap_or_else(get_project_dir)
        .join("config.toml")
}

fn get_feeds_path(base_path: impl Into<Option<PathBuf>>) -> PathBuf {
    base_path
        .into()
        .unwrap_or_else(get_project_dir)
        .join("feeds.toml")
}

fn get_project_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "freemasen", "atomizer")
        .unwrap()
        .config_dir()
        .to_path_buf()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Atom(#[from] atom_syndication::Error),
    #[error(transparent)]
    Dialog(#[from] dialoguer::Error),
    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    TomlD(#[from] toml::de::Error),
    #[error(transparent)]
    TomlS(#[from] toml::ser::Error),

    /// An unknown url scheme was provided, the scheme should be
    /// the associated value
    #[error("Unknown url scheme `{0}`")]
    UnknownScheme(String),
    #[error("Invalid file URL: `{0}`")]
    InvalidFileUrl(Url),
}
