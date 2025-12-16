use eyre::WrapErr as _;

const TIMEOUT_SECS: u64 = 300;

#[derive(serde::Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

/// Starts a local HTTP server to receive OAuth callbacks.
/// Returns the authorization code once received.
pub async fn wait_for_callback(expected_state: &str, port: u16) -> eyre::Result<String> {
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));
    let expected_state = expected_state.to_string();

    let app = axum::Router::new().route(
        "/callback",
        axum::routing::get(handle_callback(tx, expected_state)),
    );

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .wrap_err_with(|| format!("failed to bind to {addr} — is another instance running?"))?;

    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    let code = tokio::time::timeout(std::time::Duration::from_secs(TIMEOUT_SECS), rx)
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
