use core::fmt;
use std::fmt::Write;

use atom_syndication::FixedDateTime;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::Result;

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
    pub fn report_key(&self, key: &str) -> Result {
        match key {
            "feeds" => {
                let mut s = String::new();
                self.report_feeds(&mut s)?;
                println!("{s}");
            }
            _ => {
                eprintln!("Unknown key `{key}`");
                std::process::exit(1);
            }
        }
        Ok(())
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
                eprintln!("Unknown key: `{key}`");
                std::process::exit(1);
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
                eprintln!("Unknown key: `{key}`");
                std::process::exit(1);
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
}
