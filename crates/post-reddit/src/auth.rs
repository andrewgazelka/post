use eyre::WrapErr as _;

const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

/// Authenticate using Reddit's password grant flow (for "script" apps).
/// This requires a Reddit app registered as "script" type.
pub async fn authenticate(
    client_id: &str,
    client_secret: &str,
    username: &str,
    password: &str,
) -> eyre::Result<(String, Option<String>)> {
    use base64::Engine as _;

    let http = reqwest::Client::new();

    let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));

    let params = [
        ("grant_type", "password"),
        ("username", username),
        ("password", password),
    ];

    let response = http
        .post(TOKEN_URL)
        .header("Authorization", format!("Basic {credentials}"))
        .header("User-Agent", super::USER_AGENT)
        .form(&params)
        .send()
        .await
        .wrap_err("failed to authenticate with Reddit")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eyre::bail!("Reddit authentication failed ({status}): {body}");
    }

    let token: TokenResponse = response
        .json()
        .await
        .wrap_err("failed to parse Reddit token response")?;

    Ok((token.access_token, token.refresh_token))
}

/// Refresh an access token.
pub async fn refresh(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> eyre::Result<(String, Option<String>)> {
    use base64::Engine as _;

    let http = reqwest::Client::new();

    let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let response = http
        .post(TOKEN_URL)
        .header("Authorization", format!("Basic {credentials}"))
        .header("User-Agent", super::USER_AGENT)
        .form(&params)
        .send()
        .await
        .wrap_err("failed to refresh Reddit token")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        eyre::bail!("Reddit token refresh failed ({status}): {body}");
    }

    let token: TokenResponse = response
        .json()
        .await
        .wrap_err("failed to parse Reddit token response")?;

    Ok((token.access_token, token.refresh_token))
}
