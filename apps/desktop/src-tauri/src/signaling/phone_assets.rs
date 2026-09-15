use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "phone-assets/"]
struct PhoneAssets;

pub async fn serve(uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };
    let direct_asset = PhoneAssets::get(requested);
    let is_spa_fallback = direct_asset.is_none();
    let asset = direct_asset.or_else(|| PhoneAssets::get("index.html"));

    let Some(asset) = asset else {
        return (
            StatusCode::NOT_FOUND,
            "iMirror phone assets are unavailable.",
        )
            .into_response();
    };

    let content_type = if requested == "index.html" || is_spa_fallback {
        "text/html; charset=utf-8"
    } else {
        mime_guess::from_path(requested)
            .first_raw()
            .unwrap_or("application/octet-stream")
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_policy(requested))
        .header("X-Content-Type-Options", "nosniff")
        .header("Permissions-Policy", "camera=(self), microphone=()")
        .body(Body::from(asset.data.into_owned()))
        .expect("valid embedded asset response")
}

fn cache_policy(path: &str) -> &'static str {
    if path == "index.html" {
        "no-store"
    } else {
        "public, max-age=31536000, immutable"
    }
}
