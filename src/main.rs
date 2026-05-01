use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
};
use reqwest::Url;
use serde_json::Value;
use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let host = std::env::var("HOST")
        .ok()
        .and_then(|value| value.parse::<IpAddr>().ok())
        .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1]));

    let state = Arc::new(AppState {
        client: reqwest::Client::new(),
    });

    let app = Router::new().route("/{*url}", any(proxy)).with_state(state);

    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");

    println!("listening on http://{addr}");

    axum::serve(listener, app).await.expect("server failed");
}

async fn proxy(
    State(state): State<Arc<AppState>>,
    Path(raw_url): Path<String>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let target_url = match build_target_url(&raw_url, uri.query()) {
        Ok(url) => url,
        Err(message) => return (StatusCode::BAD_REQUEST, message).into_response(),
    };

    let outbound_body = patch_reasoning_content(body);
    let reqwest_method = match reqwest::Method::from_bytes(method.as_str().as_bytes()) {
        Ok(method) => method,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid method").into_response(),
    };

    let mut request = state
        .client
        .request(reqwest_method, target_url)
        .body(outbound_body);

    for (name, value) in headers.iter() {
        if should_forward_header(name.as_str()) {
            request = request.header(name.as_str(), value.as_bytes());
        }
    }

    match request.send().await {
        Ok(upstream) => response_from_upstream(upstream).await,
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            format!("upstream request failed: {}", format_error_chain(&error)),
        )
            .into_response(),
    }
}

fn format_error_chain(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();

    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }

    message
}

fn build_target_url(raw_url: &str, query: Option<&str>) -> Result<Url, &'static str> {
    let mut url = Url::parse(raw_url).map_err(|_| "url must be an absolute http(s) URL")?;

    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("url scheme must be http or https"),
    }

    if let Some(query) = query {
        url.set_query(Some(query));
    }

    Ok(url)
}

fn patch_reasoning_content(body: Bytes) -> Vec<u8> {
    let Ok(mut value) = serde_json::from_slice::<Value>(&body) else {
        return body.to_vec();
    };

    let Some(messages) = value.get_mut("messages").and_then(Value::as_array_mut) else {
        return body.to_vec();
    };

    for message in messages {
        let Some(object) = message.as_object_mut() else {
            continue;
        };

        let is_assistant = object
            .get("role")
            .and_then(Value::as_str)
            .is_some_and(|role| role == "assistant");

        if is_assistant && !object.contains_key("reasoning_content") {
            object.insert(
                "reasoning_content".to_string(),
                Value::String(String::new()),
            );
        }
    }

    serde_json::to_vec(&value).unwrap_or_else(|_| body.to_vec())
}

fn should_forward_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "content-length" | "connection" | "transfer-encoding"
    )
}

async fn response_from_upstream(upstream: reqwest::Response) -> Response {
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let body = match upstream.bytes().await {
        Ok(body) => body,
        Err(error) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("failed to read upstream response: {error}"),
            )
                .into_response();
        }
    };

    let status = StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;

    for (name, value) in headers.iter() {
        if should_forward_header(name.as_str()) {
            response.headers_mut().insert(name, value.clone());
        }
    }

    response
}
