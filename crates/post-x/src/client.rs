use eyre::WrapErr as _;

const TWEETS_URL: &str = "https://api.x.com/2/tweets";

pub struct Client {
    access_token: String,
    http: reqwest::Client,
}

#[derive(serde::Serialize)]
struct TweetRequest<'a> {
    text: &'a str,
}

#[derive(serde::Deserialize)]
pub struct TweetResponse {
    pub data: TweetData,
}

#[derive(serde::Deserialize)]
pub struct TweetData {
    pub id: String,
    pub text: String,
}

impl Client {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            http: reqwest::Client::new(),
        }
    }

    pub async fn post_tweet(&self, text: &str) -> eyre::Result<TweetResponse> {
        let request = TweetRequest { text };

        let response = self
            .http
            .post(TWEETS_URL)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .wrap_err("failed to send tweet request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eyre::bail!("tweet failed ({status}): {body}");
        }

        response
            .json()
            .await
            .wrap_err("failed to parse tweet response")
    }
}
