#[derive(clap::Parser)]
#[command(name = "post")]
#[command(about = "Post to social media from your terminal")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Text to post (shorthand for `post tweet "text"`)
    #[arg(trailing_var_arg = true)]
    text: Vec<String>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Authenticate with X (OAuth 2.0)
    Auth {
        /// Client ID from X Developer Portal
        #[arg(long, env = "CLIENT_ID")]
        client_id: String,

        /// Client Secret from X Developer Portal
        #[arg(long, env = "CLIENT_SECRET")]
        client_secret: String,
    },
    /// Post a tweet
    Post {
        /// Text to post
        text: String,
    },
    /// Show current auth status
    Status,
    /// Clear saved credentials
    Logout,
}

pub async fn run() -> eyre::Result<()> {
    use clap::Parser as _;
    let cli = Cli::parse();

    // If text is provided directly, treat as post command
    if !cli.text.is_empty() {
        let text = cli.text.join(" ");
        return post_tweet(&text).await;
    }

    match cli.command {
        Some(Command::Auth {
            client_id,
            client_secret,
        }) => auth(&client_id, &client_secret).await,
        Some(Command::Post { text }) => post_tweet(&text).await,
        Some(Command::Status) => status(),
        Some(Command::Logout) => logout(),
        None => {
            use clap::CommandFactory as _;
            Cli::command().print_help()?;
            Ok(())
        }
    }
}

async fn auth(client_id: &str, client_secret: &str) -> eyre::Result<()> {
    let oauth = crate::auth::OAuth2Client::new(client_id.to_string(), client_secret.to_string());

    let (access_token, refresh_token) = oauth.authorize().await?;

    let mut config = crate::config::Config::load()?;
    config.client_id = Some(client_id.to_string());
    config.client_secret = Some(client_secret.to_string());
    config.access_token = Some(access_token);
    config.refresh_token = refresh_token;
    config.save()?;

    println!("Authentication successful!");
    Ok(())
}

async fn post_tweet(text: &str) -> eyre::Result<()> {
    let mut config = crate::config::Config::load()?;

    let access_token = config
        .access_token
        .as_ref()
        .ok_or_else(|| eyre::eyre!("not authenticated — run `post auth` first"))?
        .clone();

    let client = crate::twitter::Client::new(access_token);

    match client.post_tweet(text).await {
        Ok(response) => {
            println!("Posted: https://x.com/i/status/{}", response.data.id);
            Ok(())
        }
        Err(e) => {
            // Try to refresh token if we have one
            if let (Some(client_id), Some(client_secret), Some(refresh_token)) = (
                &config.client_id,
                &config.client_secret,
                &config.refresh_token,
            ) {
                tracing::debug!("attempting token refresh");
                let oauth =
                    crate::auth::OAuth2Client::new(client_id.clone(), client_secret.clone());
                if let Ok((new_access, new_refresh)) = oauth.refresh(refresh_token).await {
                    config.access_token = Some(new_access.clone());
                    config.refresh_token = new_refresh;
                    config.save()?;

                    // Retry with new token
                    let client = crate::twitter::Client::new(new_access);
                    let response = client.post_tweet(text).await?;
                    println!("Posted: https://x.com/i/status/{}", response.data.id);
                    return Ok(());
                }
            }
            Err(e)
        }
    }
}

fn status() -> eyre::Result<()> {
    let config = crate::config::Config::load()?;

    if config.access_token.is_some() {
        println!("Authenticated");
        if config.refresh_token.is_some() {
            println!("Refresh token: saved");
        }
    } else {
        println!("Not authenticated — run `post auth` first");
    }

    Ok(())
}

fn logout() -> eyre::Result<()> {
    let mut config = crate::config::Config::load()?;
    config.access_token = None;
    config.refresh_token = None;
    config.save()?;
    println!("Logged out");
    Ok(())
}
