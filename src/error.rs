use url::Url;

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
    #[error("Unknown key: `{0}`")]
    UnknownKey(String),
    #[error("Invalid arguments, {0}")]
    InvalidArgument(String),
    #[error("Missing feed: {0}")]
    MissingFeed(String),
    #[error("Previously setup, use --force (-f) to overwrite exiting config")]
    PreviouslySetup,
}
