#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    cli::run().await
}

mod cli {
    use clap::Parser as _;

    #[derive(clap::Parser)]
    #[command(name = "post")]
    #[command(about = "Post to social media from your terminal")]
    #[command(version)]
    struct Cli {
        #[command(subcommand)]
        command: Command,
    }

    #[derive(clap::Subcommand)]
    enum Command {
        /// Post to X (Twitter)
        X {
            #[command(subcommand)]
            command: XCommand,
        },
        /// Post to Reddit
        Reddit {
            #[command(subcommand)]
            command: RedditCommand,
        },
    }

    #[derive(clap::Subcommand)]
    enum XCommand {
        /// Authenticate with X (OAuth 2.0)
        Auth {
            /// Client ID from X Developer Portal
            #[arg(long, env = "X_CLIENT_ID")]
            client_id: String,

            /// Client Secret from X Developer Portal
            #[arg(long, env = "X_CLIENT_SECRET")]
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

    #[derive(clap::Subcommand)]
    enum RedditCommand {
        /// Authenticate with Reddit
        Auth {
            /// Client ID from Reddit (reddit.com/prefs/apps)
            #[arg(long, env = "REDDIT_CLIENT_ID")]
            client_id: String,

            /// Client Secret from Reddit
            #[arg(long, env = "REDDIT_CLIENT_SECRET")]
            client_secret: String,

            /// Reddit username
            #[arg(long, env = "REDDIT_USERNAME")]
            username: String,

            /// Reddit password
            #[arg(long, env = "REDDIT_PASSWORD")]
            password: String,
        },
        /// Submit a post to a subreddit
        Post {
            /// Target subreddit (without r/)
            #[arg(short, long)]
            subreddit: String,

            /// Post title
            #[arg(short, long)]
            title: String,

            /// Post body text (optional for self posts)
            #[arg(short, long)]
            body: Option<String>,

            /// URL to submit as link post (mutually exclusive with body)
            #[arg(short, long, conflicts_with = "body")]
            link: Option<String>,
        },
        /// Show current auth status
        Status,
        /// Clear saved credentials
        Logout,
    }

    pub async fn run() -> eyre::Result<()> {
        let cli = Cli::parse();

        match cli.command {
            Command::X { command } => handle_x(command).await,
            Command::Reddit { command } => handle_reddit(command).await,
        }
    }

    async fn handle_x(command: XCommand) -> eyre::Result<()> {
        match command {
            XCommand::Auth {
                client_id,
                client_secret,
            } => {
                post_x::authenticate(&client_id, &client_secret).await?;
                println!("Authentication successful!");
                Ok(())
            }
            XCommand::Post { text } => {
                let result = post_x::post(&text).await?;
                println!("Posted: {}", result.url);
                Ok(())
            }
            XCommand::Status => {
                match post_x::status() {
                    Some(msg) => println!("{msg}"),
                    None => println!("Not authenticated with X — run `post x auth` first"),
                }
                Ok(())
            }
            XCommand::Logout => {
                post_x::logout()?;
                println!("Logged out of X");
                Ok(())
            }
        }
    }

    async fn handle_reddit(command: RedditCommand) -> eyre::Result<()> {
        match command {
            RedditCommand::Auth {
                client_id,
                client_secret,
                username,
                password,
            } => {
                post_reddit::auth(&client_id, &client_secret, &username, &password).await?;
                println!("Authentication successful!");
                Ok(())
            }
            RedditCommand::Post {
                subreddit,
                title,
                body,
                link,
            } => {
                let result = if let Some(url) = link {
                    post_reddit::post_link(&subreddit, &title, &url).await?
                } else {
                    post_reddit::post(&subreddit, &title, body.as_deref()).await?
                };
                println!("Posted: {}", result.url);
                Ok(())
            }
            RedditCommand::Status => {
                match post_reddit::status() {
                    Some(msg) => println!("{msg}"),
                    None => println!("Not authenticated with Reddit — run `post reddit auth` first"),
                }
                Ok(())
            }
            RedditCommand::Logout => {
                post_reddit::logout()?;
                println!("Logged out of Reddit");
                Ok(())
            }
        }
    }
}
