mod auth;
mod client;

pub use auth::authenticate;
pub use client::Client;

const USER_AGENT: &str = concat!("post-cli/", env!("CARGO_PKG_VERSION"), " (by /u/andrewgazelka)");

/// Authenticate with Reddit using password grant flow (for "script" apps).
pub async fn auth(
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> eyre::Result<post_core::Config> {
    let (access_token, refresh_token) =
        auth::authenticate(client_id, client_secret, username, password).await?;

    let mut config = post_core::Config::load()?;
    config.reddit = Some(post_core::RedditConfig {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        username: username.to_string(),
        access_token,
        refresh_token,
    });
    config.save()?;

    Ok(config)
}

/// Submit a text post to a subreddit.
pub async fn post(subreddit: &str, title: &str, text: Option<&str>) -> eyre::Result<post_core::PostResult> {
    let mut config = post_core::Config::load()?;

    let reddit_config = config
        .reddit
        .as_ref()
        .ok_or_else(|| eyre::eyre!("not authenticated with Reddit — run `post reddit auth` first"))?
        .clone();

    let client = Client::new(reddit_config.access_token.clone());

    match client.submit_self_post(subreddit, title, text).await {
        Ok(url) => Ok(post_core::PostResult { url }),
        Err(e) => {
            // Try token refresh
            if let Some(refresh_token) = &reddit_config.refresh_token {
                tracing::debug!("attempting token refresh");

                if let Ok((new_access, new_refresh)) = auth::refresh(
                    &reddit_config.client_id,
                    &reddit_config.client_secret,
                    refresh_token,
                )
                .await
                {
                    config.reddit = Some(post_core::RedditConfig {
                        client_id: reddit_config.client_id,
                        client_secret: reddit_config.client_secret,
                        username: reddit_config.username,
                        access_token: new_access.clone(),
                        refresh_token: new_refresh,
                    });
                    config.save()?;

                    let client = Client::new(new_access);
                    let url = client.submit_self_post(subreddit, title, text).await?;
                    return Ok(post_core::PostResult { url });
                }
            }
            Err(e)
        }
    }
}

/// Submit a link post to a subreddit.
pub async fn post_link(subreddit: &str, title: &str, url: &str) -> eyre::Result<post_core::PostResult> {
    let mut config = post_core::Config::load()?;

    let reddit_config = config
        .reddit
        .as_ref()
        .ok_or_else(|| eyre::eyre!("not authenticated with Reddit — run `post reddit auth` first"))?
        .clone();

    let client = Client::new(reddit_config.access_token.clone());

    match client.submit_link_post(subreddit, title, url).await {
        Ok(post_url) => Ok(post_core::PostResult { url: post_url }),
        Err(e) => {
            // Try token refresh
            if let Some(refresh_token) = &reddit_config.refresh_token {
                tracing::debug!("attempting token refresh");

                if let Ok((new_access, new_refresh)) = auth::refresh(
                    &reddit_config.client_id,
                    &reddit_config.client_secret,
                    refresh_token,
                )
                .await
                {
                    config.reddit = Some(post_core::RedditConfig {
                        client_id: reddit_config.client_id,
                        client_secret: reddit_config.client_secret,
                        username: reddit_config.username,
                        access_token: new_access.clone(),
                        refresh_token: new_refresh,
                    });
                    config.save()?;

                    let client = Client::new(new_access);
                    let post_url = client.submit_link_post(subreddit, title, url).await?;
                    return Ok(post_core::PostResult { url: post_url });
                }
            }
            Err(e)
        }
    }
}

/// Check authentication status
pub fn status() -> Option<String> {
    let config = post_core::Config::load().ok()?;
    let reddit = config.reddit.as_ref()?;
    Some(format!("Authenticated with Reddit as u/{}", reddit.username))
}

/// Clear Reddit credentials
pub fn logout() -> eyre::Result<()> {
    let mut config = post_core::Config::load()?;
    config.reddit = None;
    config.save()?;
    Ok(())
}
