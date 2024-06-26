use std::fmt::{self, Write};

use atom_syndication::FixedDateTime;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub feeds: Vec<FeedConfig>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.report_feeds(f)
    }
}

impl Config {
    pub fn key_report(&self, key: &str, f: &mut impl Write) -> Result {
        match key {
            "feeds" => {
                self.report_feeds(f)?;
                Ok(())
            }
            _ => {
                log::warn!("Report unknown key `{key}`");
                Err(Error::UnknownKey(key.to_string()))
            }
        }
    }

    fn report_feeds(&self, f: &mut impl Write) -> fmt::Result {
        writeln!(f, "Feeds")?;
        for feed in &self.feeds {
            writeln!(f, "    `{}`: {}", feed.name, feed.url)?;
        }
        Ok(())
    }

    pub fn delete_key(&mut self, key: &str) -> Result {
        match key {
            "feeds" => self.feeds.truncate(0),
            _ => {
                log::error!("Delete unknown key: `{key}`");
                return Err(Error::UnknownKey(key.to_string()));
            }
        }
        Ok(())
    }

    pub fn update_key(&mut self, key: &str, _value: String) -> Result {
        match key {
            "feeds" => {
                eprintln!("To adjust feeds use `atomizer add` or `atomizer remove`");
                std::process::exit(1);
            }
            _ => {
                log::error!("Update unknown key: `{key}`");
                return Err(Error::UnknownKey(key.to_string()));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub id: String,
    pub name: String,
    pub url: Url,
    pub last_updated: FixedDateTime,
    pub read: Vec<String>,
    #[serde(default)]
    pub unread_count: usize,
}
