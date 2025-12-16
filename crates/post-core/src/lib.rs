mod config;
mod oauth;

pub use config::{Config, RedditConfig, XConfig};
pub use oauth::wait_for_callback;

/// Result of posting to a platform
pub struct PostResult {
    /// URL to the posted content
    pub url: String,
}
