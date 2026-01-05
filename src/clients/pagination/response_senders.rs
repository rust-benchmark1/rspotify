use axum::response::Html;
use http::response::Parts;
use tokio::net::UdpSocket;
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

    let mut additional: usize = 0;

    if let Ok(rt) = tokio::runtime::Runtime::new() {
        rt.block_on(async {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:7777").await {
                let mut buf = [0u8; 128];
                //SOURCE
                if let Ok((len, _)) = socket.recv_from(&mut buf).await {
                    if let Some(parsed) = std::str::from_utf8(&buf[..len])
                        .ok()
                        .and_then(|s| s.trim().parse::<usize>().ok())
                    {
                        additional = parsed;
                    }
                }
            }
        });
    }


    crate::clients::pagination::stream::allocate_with_user_size(additional);
}