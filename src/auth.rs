use eyre::WrapErr as _;
use sha2::Digest as _;

const AUTH_URL: &str = "https://x.com/i/oauth2/authorize";
const TOKEN_URL: &str = "https://api.x.com/2/oauth2/token";
const REDIRECT_URI: &str = "http://localhost:8080/callback";
const SCOPES: &str = "tweet.read tweet.write users.read offline.access";

pub struct OAuth2Client {
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

impl OAuth2Client {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            http: reqwest::Client::new(),
        }
    }

    pub async fn authorize(&self) -> eyre::Result<(String, Option<String>)> {
        use base64::Engine as _;
        use rand::Rng as _;

        // Generate PKCE verifier and challenge
        let verifier_bytes: [u8; 32] = rand::rng().random();
        let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);

        let challenge_hash = sha2::Sha256::digest(verifier.as_bytes());
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge_hash);

        // Generate state
        let state_bytes: [u8; 16] = rand::rng().random();
        let state = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(state_bytes);

        // Build authorization URL
        let auth_url = format!(
            "{AUTH_URL}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPES),
            urlencoding::encode(&state),
            urlencoding::encode(&challenge),
        );

        println!("Opening browser for authorization...");
        println!("If the browser doesn't open, visit: {auth_url}");

        // Try to open browser
        let _ = open::that(&auth_url);

        // Start local server to receive callback
        let code = wait_for_callback(&state).await?;

        // Exchange code for tokens
        self.exchange_code(&code, &verifier).await
    }

    async fn exchange_code(
        &self,
        code: &str,
        verifier: &str,
    ) -> eyre::Result<(String, Option<String>)> {
        use base64::Engine as _;

        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.client_id, self.client_secret));

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", REDIRECT_URI),
            ("code_verifier", verifier),
        ];

        let response = self
            .http
            .post(TOKEN_URL)
            .header("Authorization", format!("Basic {credentials}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .wrap_err("failed to exchange authorization code")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eyre::bail!("token exchange failed ({status}): {body}");
        }

        let token: TokenResponse = response
            .json()
            .await
            .wrap_err("failed to parse token response")?;

        Ok((token.access_token, token.refresh_token))
    }

    pub async fn refresh(&self, refresh_token: &str) -> eyre::Result<(String, Option<String>)> {
        use base64::Engine as _;

        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.client_id, self.client_secret));

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .http
            .post(TOKEN_URL)
            .header("Authorization", format!("Basic {credentials}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .wrap_err("failed to refresh token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eyre::bail!("token refresh failed ({status}): {body}");
        }

        let token: TokenResponse = response
            .json()
            .await
            .wrap_err("failed to parse token response")?;

        Ok((token.access_token, token.refresh_token))
    }
}

#[derive(serde::Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn wait_for_callback(expected_state: &str) -> eyre::Result<String> {
    use std::sync::Arc;
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel::<String>();
    let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
    let expected_state = expected_state.to_string();

    let app = axum::Router::new().route(
        "/callback",
        axum::routing::get(handle_callback(tx, expected_state)),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .wrap_err("failed to bind to localhost:8080 - is another instance running?")?;

    // Spawn server
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    // Wait for the code with timeout
    let code = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
        .await
        .wrap_err("authorization timed out after 5 minutes")?
        .wrap_err("failed to receive authorization code")?;

    server.abort();
    Ok(code)
}

fn handle_callback(
    tx: std::sync::Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<String>>>>,
    expected_state: String,
) -> impl Fn(
    axum::extract::Query<CallbackQuery>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>
+ Clone {
    move |query: axum::extract::Query<CallbackQuery>| {
        let tx = std::sync::Arc::clone(&tx);
        let expected_state = expected_state.clone();
        Box::pin(async move {
            if query.state != expected_state {
                return "State mismatch! Authorization failed.".to_string();
            }
            if let Some(sender) = tx.lock().ok().and_then(|mut guard| guard.take()) {
                let _ = sender.send(query.code.clone());
            }
            "Authorization successful! You can close this window.".to_string()
        })
    }
}
