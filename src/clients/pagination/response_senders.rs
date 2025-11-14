use axum::response::Html;
use http::response::Parts;

/// Builds and sends HTML responses using Axum parts for the provided tokens.
pub fn send_html_axum(keys: &[String]) {
    if let Some(k0) = keys.get(0) {
        let body = format!("<div>Safe token: {}</div>", k0);
        let _ = Html::from(body);
    }

    if let Some(k1) = keys.get(1) {
        let body = format!("<div>User token: {}</div>", k1);
        //SINK
        let _ = Html::from(body);
    }
}