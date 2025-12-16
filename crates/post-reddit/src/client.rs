use eyre::WrapErr as _;

const SUBMIT_URL: &str = "https://oauth.reddit.com/api/submit";

pub struct Client {
    access_token: String,
    http: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct SubmitResponse {
    json: SubmitJson,
}

#[derive(serde::Deserialize)]
struct SubmitJson {
    errors: Vec<Vec<String>>,
    data: Option<SubmitData>,
}

#[derive(serde::Deserialize)]
struct SubmitData {
    url: String,
}

impl Client {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            http: reqwest::Client::new(),
        }
    }

    /// Submit a self (text) post to a subreddit.
    pub async fn submit_self_post(
        &self,
        subreddit: &str,
        title: &str,
        text: Option<&str>,
    ) -> eyre::Result<String> {
        let mut params = vec![
            ("api_type", "json"),
            ("kind", "self"),
            ("sr", subreddit),
            ("title", title),
        ];

        if let Some(text) = text {
            params.push(("text", text));
        }

        self.submit(&params).await
    }

    /// Submit a link post to a subreddit.
    pub async fn submit_link_post(
        &self,
        subreddit: &str,
        title: &str,
        url: &str,
    ) -> eyre::Result<String> {
        let params = [
            ("api_type", "json"),
            ("kind", "link"),
            ("sr", subreddit),
            ("title", title),
            ("url", url),
        ];

        self.submit(&params).await
    }

    async fn submit(&self, params: &[(&str, &str)]) -> eyre::Result<String> {
        let response = self
            .http
            .post(SUBMIT_URL)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", super::USER_AGENT)
            .form(params)
            .send()
            .await
            .wrap_err("failed to submit Reddit post")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            eyre::bail!("Reddit submission failed ({status}): {body}");
        }

        let submit: SubmitResponse = response
            .json()
            .await
            .wrap_err("failed to parse Reddit submit response")?;

        if !submit.json.errors.is_empty() {
            let errors: Vec<String> = submit
                .json
                .errors
                .iter()
                .map(|e| e.join(": "))
                .collect();
            eyre::bail!("Reddit submission errors: {}", errors.join(", "));
        }

        let data = submit
            .json
            .data
            .ok_or_else(|| eyre::eyre!("Reddit submission succeeded but returned no URL"))?;

        Ok(data.url)
    }
}
