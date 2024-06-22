use clap::Parser;
use url::Url;

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
    },
    /// Add a feed
    Add {
        url: Url,
    },
    /// Remove a feed
    Remove {
        id_or_name: String,
    },
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
async fn main() -> Result<(), atomizer::Error> {
    env_logger::init();
    let args = Args::parse();
    match args {
        Args::Read { no_update, category, all } => {
            if !no_update {
                atomizer::run_update().await?;
            }
            atomizer::run_read(category, all).await?;
        }
        Args::Categories => atomizer::run_categories().await?,
        Args::Update => atomizer::run_update().await?,
        Args::Setup { force } => atomizer::run_setup(force).await?,
        Args::Add { url } => atomizer::run_add(url).await?,
        Args::Remove { id_or_name } => atomizer::run_remove(id_or_name).await?,
        Args::Config { delete, key, value } => atomizer::run_config(delete, key, value).await?,
    }
    Ok(())
}
