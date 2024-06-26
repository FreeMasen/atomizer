use std::collections::HashMap;

use atom_syndication::Entry;
use atomizer::{Config, Feeds};
use chrono::Duration;
use clap::Parser;
use pager_rs::{CommandList, State, StatusBar};
use url::Url;

type Result<T = (), E = atomizer::Error> = std::result::Result<T, E>;

#[derive(Debug, Parser)]
pub enum Args {
    /// Read your feeds
    Read {
        /// Skip re-fetching the feed
        #[clap(long, short)]
        no_update: bool,
        /// Filter entries by a category
        #[clap(long, short)]
        category: Option<String>,
        /// Include previously read entries
        #[clap(long, short)]
        all: bool,
    },
    /// Query the categories in your feeds
    Categories,
    /// Update the feeds you've added
    Update,
    /// Setup the data directories
    Setup {
        #[clap(long, short)]
        force: bool,
        #[clap(long, short = 'p')]
        get_path: bool,
    },
    /// Add a feed
    Add { url: Url },
    /// Remove a feed
    Remove { id_or_name: String },
    /// Interact with Configuration
    Config {
        /// The provided key will be removed
        #[clap(long, short)]
        #[arg(conflicts_with("value"))]
        delete: bool,
        /// If a value is provided, the key to assign the value to
        /// if no value is provided print the configuration key's value
        #[arg(required_if_eq("delete", "true"))]
        key: Option<String>,
        /// The value to assign to the key
        value: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result {
    env_logger::init();
    let args = Args::parse();
    match args {
        Args::Read {
            no_update,
            category,
            all,
        } => {
            if !no_update {
                run_update().await?;
            }
            run_read(category, all).await?;
        }
        Args::Categories => run_categories().await?,
        Args::Update => run_update().await?,
        Args::Setup { force, get_path } => {
            if get_path {
                println!("{}", atomizer::get_project_dir().display());
            } else {
                run_setup(force).await?
            }
        },
        Args::Add { url } => run_add(url).await?,
        Args::Remove { id_or_name } => run_remove(id_or_name).await?,
        Args::Config { delete, key, value } => run_config(delete, key, value).await?,
    }
    Ok(())
}

pub async fn run_update() -> Result {
    let mut config = atomizer::get_config().await?;
    let mut feeds = atomizer::get_feeds().await?;
    for feed in config.feeds.iter_mut() {
        let feed2 = atomizer::fetch_feed(feed.url.clone()).await?;
        if let Some(existing_feed) = feeds.feeds.iter_mut().find(|f| f.id == feed.id) {
            // only clear old read values on updates older than an hour
            if feed
                .last_updated
                .signed_duration_since(&feed2.updated)
                .abs()
                > Duration::hours(1)
            {
                feed.read
                    .retain(|id| feed2.entries().iter().any(|e| &e.id == id));
            }
            feed.last_updated = feed2.updated;
            // eprintln!("{} <-> {}", feed2.entries.len(), feed.read.len());
            feed.unread_count = feed2.entries.len().saturating_sub(feed.read.len());
            *existing_feed = feed2;
        } else {
            feeds.feeds.push(feed2);
        }
    }
    atomizer::save_config(&config, None).await?;
    atomizer::save_feeds(&feeds, None).await?;
    Ok(())
}

pub async fn run_read(category_filter: Option<String>, all: bool) -> Result {
    let mut config = atomizer::get_config().await?;
    if config.feeds.is_empty() {
        println!("No feeds to read");
        return Ok(());
    }
    'feeds: loop {
        let idx = dialoguer::Select::new()
            .items(
                &config
                    .feeds
                    .iter()
                    .map(|f| format!("{} ({})", f.name, f.unread_count))
                    .collect::<Vec<_>>(),
            )
            .clear(false)
            .max_length(10)
            .default(0)
            .interact_opt()?;
        let Some(feed_idx) = idx else {
            break 'feeds;
        };
        let feeds = atomizer::get_feeds().await?;
        if let Some(feed_cfg) = config.feeds.get_mut(feed_idx) {
            let Some(feed) = feeds.feeds.iter().find(|f| f.id == feed_cfg.id) else {
                break 'feeds;
            };
            'entries: loop {
                let iter = feed.entries.iter();
                let unread_entries: Vec<_> = if let Some(cat) = category_filter.as_ref() {
                    iter.filter(|e: &&Entry| {
                        e.categories.iter().any(|c| c.term.starts_with(cat))
                            && !feed_cfg.read.contains(&e.id)
                    })
                    .collect()
                } else {
                    iter.filter(|e: &&Entry| {
                        let was_read = feed_cfg.read.contains(&e.id);
                        if all {
                            return true;
                        }
                        !was_read
                    })
                    .collect()
                };
                if unread_entries.is_empty() {
                    break 'entries;
                }
                let unread_titles: Vec<String> =
                    unread_entries.iter().map(|e| e.title.to_string()).collect();
                let selected_entry = dialoguer::Select::new()
                    .with_prompt("Unread")
                    .items(&unread_titles)
                    .clear(false)
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
            atomizer::save_config(&config, None).await.ok();
            return Err(atomizer::Error::MissingFeed(format!(
                "feed idx not found: {feed_idx} len: {config:#?}"
            )));
        }
    }
    atomizer::save_config(&config, None).await
}

pub async fn run_categories() -> Result {
    let feeds = atomizer::get_feeds().await?;
    let mut categories: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for feed in feeds.feeds.iter() {
        for entry in feed.entries() {
            for cat in entry.categories() {
                categories
                    .entry(cat.term.to_string())
                    .and_modify(|v| {
                        v.entry(feed.title.to_string())
                            .and_modify(|f: &mut usize| {
                                *f += 1;
                            })
                            .or_insert_with(|| 1);
                    })
                    .or_insert_with(|| HashMap::from_iter([(feed.title.to_string(), 1)]));
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
    let proj_dir = atomizer::get_project_dir();
    let config_path = atomizer::get_config_path(proj_dir.clone());
    let feeds_path = atomizer::get_feeds_path(proj_dir);
    if !force && (config_path.exists() || feeds_path.exists()) {
        return Err(atomizer::Error::PreviouslySetup);
    }
    let default_config = Config::default();
    let default_feeds = Feeds::default();
    atomizer::save_config(&default_config, config_path).await?;
    atomizer::save_feeds(&default_feeds, feeds_path).await?;
    Ok(())
}

pub async fn run_add(url: Url) -> Result {
    let mut config = atomizer::get_config().await?;
    let mut feeds = atomizer::get_feeds().await?;
    let feed = atomizer::fetch_feed(url.clone()).await?;
    config.feeds.push(atomizer::FeedConfig {
        id: feed.id.clone(),
        name: feed.title.to_string(),
        url,
        last_updated: feed.updated,
        read: Vec::new(),
        unread_count: feed.entries.len(),
    });
    feeds.feeds.push(feed);
    atomizer::save_config(&config, None).await?;
    atomizer::save_feeds(&feeds, None).await?;
    Ok(())
}

pub async fn run_remove(id_or_name: String) -> Result {
    let mut config = atomizer::get_config().await?;
    let mut feeds = atomizer::get_feeds().await?;
    let Some(feed_idx) = config.feeds.iter().enumerate().find_map(|(idx, f)| {
        if f.id == id_or_name || f.name == id_or_name {
            return Some(idx);
        }
        None
    }) else {
        return Err(atomizer::Error::MissingFeed(format!("`{id_or_name}`")));
    };
    let feed = config.feeds.remove(feed_idx);
    feeds.feeds.retain(|f| f.id != feed.id);
    atomizer::save_config(&config, None).await?;
    atomizer::save_feeds(&feeds, None).await?;
    Ok(())
}

async fn run_config(delete: bool, key: Option<String>, value: Option<String>) -> Result {
    let mut config = atomizer::get_config().await?;
    match (key, value) {
        (Some(key), Some(value)) => {
            config.update_key(&key, value)?;
        }
        (Some(key), None) => {
            if delete {
                config.delete_key(&key)?;
            } else {
                let mut s = String::new();
                config.key_report(&key, &mut s)?;
                println!("{s}");
            }
        }
        (None, Some(_)) => {
            return Err(atomizer::Error::InvalidArgument("no key value".to_string()));
        }
        (None, None) => {
            println!("{config}");
            return Ok(());
        }
    }
    atomizer::save_config(&config, None).await
}
