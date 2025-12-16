mod auth;
mod client;

pub use auth::OAuth2Client;
pub use client::Client;

const CALLBACK_PORT: u16 = 8080;

pub fn redirect_uri() -> String {
    format!("http://localhost:{CALLBACK_PORT}/callback")
}

/// Authenticate with X/Twitter using OAuth2 PKCE flow.
pub async fn authenticate(client_id: &str, client_secret: &str) -> eyre::Result<post_core::Config> {
    let oauth = OAuth2Client::new(client_id.to_string(), client_secret.to_string());
    let (access_token, refresh_token) = oauth.authorize().await?;

    let mut config = post_core::Config::load()?;
    config.x = Some(post_core::XConfig {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        access_token,
        refresh_token,
    });
    config.save()?;

    Ok(config)
}

/// Post a tweet. Handles token refresh automatically.
pub async fn post(text: &str) -> eyre::Result<post_core::PostResult> {
    let mut config = post_core::Config::load()?;

    let x_config = config
        .x
        .as_ref()
        .ok_or_else(|| eyre::eyre!("not authenticated with X — run `post x auth` first"))?
        .clone();

    let client = Client::new(x_config.access_token.clone());

    match client.post_tweet(text).await {
        Ok(response) => Ok(post_core::PostResult {
            url: format!("https://x.com/i/status/{}", response.data.id),
        }),
        Err(e) => {
            // Try token refresh
            if let Some(refresh_token) = &x_config.refresh_token {
                tracing::debug!("attempting token refresh");
                let oauth =
                    OAuth2Client::new(x_config.client_id.clone(), x_config.client_secret.clone());

                if let Ok((new_access, new_refresh)) = oauth.refresh(refresh_token).await {
                    config.x = Some(post_core::XConfig {
                        client_id: x_config.client_id,
                        client_secret: x_config.client_secret,
                        access_token: new_access.clone(),
                        refresh_token: new_refresh,
                    });
                    config.save()?;

                    let client = Client::new(new_access);
                    let response = client.post_tweet(text).await?;
                    return Ok(post_core::PostResult {
                        url: format!("https://x.com/i/status/{}", response.data.id),
                    });
                }
            }
            Err(e)
        }
    }
}

/// Check authentication status
pub fn status() -> Option<String> {
    let config = post_core::Config::load().ok()?;
    config.x.as_ref()?;
    Some("Authenticated with X".to_string())
}

/// Clear X credentials
pub fn logout() -> eyre::Result<()> {
    let mut config = post_core::Config::load()?;
    config.x = None;
    config.save()?;
    Ok(())
}
