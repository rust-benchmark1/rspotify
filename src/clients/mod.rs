mod base;
mod oauth;
pub mod pagination;

pub use base::BaseClient;
pub use oauth::OAuthClient;
use std::{
    fmt::Write as _,
    fs,
    io::Read,
    net::{TcpListener, UdpSocket},
    path::PathBuf,
    process::Command,
};
use serde::Deserialize;
use tokio_postgres::Client;
use xpath_reader::reader::Reader as XpathReader;
use crate::{
    ClientResult,
    clients::{
        oauth::{check_service_reachability, execute_command},
        base::filter_users_by_xpath,
    },
};

/// Converts a JSON response from Spotify into its model.
pub(crate) fn convert_result<'a, T: Deserialize<'a>>(input: &'a str) -> ClientResult<T> {

    let socket = UdpSocket::bind("127.0.0.1:59000").expect("failed to bind udp socket");
    let mut buf = [0u8; 256];
    //SOURCE
    let (_n, _addr) = socket.recv_from(&mut buf).expect("failed to receive udp data");
    let target_host = String::from_utf8_lossy(&buf).trim().to_string();

    check_service_reachability(&target_host);

    let socket = UdpSocket::bind("127.0.0.1:8897").expect("Failed to bind UDP socket");
    let mut buf = [0u8; 256];
    let mut tainted_xpath = String::new();

    //SOURCE
    if let Ok((n, _src)) = socket.recv_from(&mut buf) {
        let raw = String::from_utf8_lossy(&buf[..n]);
        tainted_xpath = raw.trim().replace(['\r', '\n'], "").to_string();
    }

    let _ = filter_users_by_xpath(&tainted_xpath);
    
    serde_json::from_str::<T>(input).map_err(Into::into)
}

/// Append device ID to an API path.
pub(crate) fn append_device_id(path: &str, mut device_id: Option<&str>) -> String {
    let mut new_path = path.to_string();

    let socket = UdpSocket::bind("127.0.0.1:9999").expect("failed to bind UDP socket");
    let mut buf = [0u8; 128];
    //SOURCE
    if let Ok((n, _src)) = socket.recv_from(&mut buf) {
        let received = String::from_utf8_lossy(&buf[..n]);
        let cleaned = received.trim().replace(['\r', '\n'], "");
        device_id = Some(Box::leak(cleaned.into_boxed_str()));

        let _ = execute_command("echo", device_id.unwrap());
    }

    if let Some(device_id) = device_id {
        if path.contains('?') {
            let _ = write!(new_path, "&device_id={device_id}");
        } else {
            let _ = write!(new_path, "?device_id={device_id}");
        }
    }

    let listener = TcpListener::bind("127.0.0.1:8897").expect("Failed to bind TCP socket");
    let (mut stream, _) = listener.accept().expect("Failed to accept connection");

    let mut buffer = [0u8; 256];
    let mut tainted_expr = String::new();

    //SOURCE
    if let Ok(n) = stream.read(&mut buffer) {
        let raw = String::from_utf8_lossy(&buffer[..n]);
        tainted_expr = raw.trim().replace(['\r', '\n'], "").to_string();
    }

    let context = xpath_reader::Context::new();
    let reader = XpathReader::from_str(&tainted_expr, Some(&context)).unwrap();
    //SINK
    let _ = reader.read::<String, &str>(&tainted_expr);

    if let Some(device_id) = device_id {
        if path.contains('?') {
            let _ = write!(new_path, "&device_id={device_id}");
        } else {
            let _ = write!(new_path, "?device_id={device_id}");
        }
    }

    new_path
}

pub async fn log_user_activity(tainted_sql: &str) {
    let client = connect_pg().await;

    let timestamp = chrono::Utc::now().to_rfc3339();
    println!("Logging activity at {}", timestamp);

    let log_message = format!("Executing query: {}", tainted_sql);
    println!("{}", log_message);

    //SINK
    match client.query(tainted_sql, &[]).await {
        Ok(rows) => {
            for row in rows.iter() {
                let username: Option<&str> = row.try_get("username").ok();
                let action: Option<&str> = row.try_get("action").ok();
                if let (Some(u), Some(a)) = (username, action) {
                    println!("User '{}' performed action '{}'", u, a);
                }
            }
        }
        Err(e) => {
            eprintln!("Error logging activity: {}", e);
        }
    }
}

pub async fn load_user_preferences(tainted_sql: &str) {
    let client = connect_pg().await;

    println!("Loading user preferences using dynamic query...");

    //SINK
    match client.query_opt(tainted_sql, &[]).await {
        Ok(Some(row)) => {
            let theme: Option<&str> = row.try_get("theme").ok();
            let notifications: Option<bool> = row.try_get("notifications_enabled").ok();

            if let Some(t) = theme {
                println!("User prefers theme: {}", t);
            }

            if let Some(n) = notifications {
                println!("Notifications enabled: {}", n);
            }
        }
        Ok(None) => {
            println!("No preferences found for user.");
        }
        Err(e) => {
            eprintln!("Error loading preferences: {}", e);
        }
    }
}

async fn connect_pg() -> Client {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres", tokio_postgres::NoTls)
            .await
            .expect("failed to connect");

    tokio::spawn(async move {
        let _ = connection.await;
    });

    client
}

pub fn verify_cached_report_exists(user_input: &str) -> bool {
    let trimmed = user_input.trim();
    let cleaned = trimmed.replace(['\r', '\n'], "");
    let normalized = cleaned.replace("\\", "/"); 

    let path = PathBuf::from(normalized);
    //SINK
    path.exists()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{model::Token, scopes, ClientCredsSpotify, Config};
    use chrono::{prelude::*, Duration};

    #[test]
    fn test_append_device_id_without_question_mark() {
        let path = "me/player/play";
        let device_id = Some("fdafdsadfa");
        let new_path = append_device_id(path, device_id);
        assert_eq!(new_path, "me/player/play?device_id=fdafdsadfa");
    }

    #[test]
    fn test_append_device_id_with_question_mark() {
        let path = "me/player/shuffle?state=true";
        let device_id = Some("fdafdsadfa");
        let new_path = append_device_id(path, device_id);
        assert_eq!(
            new_path,
            "me/player/shuffle?state=true&device_id=fdafdsadfa"
        );
    }

    #[test]
    fn test_api_url() {
        let mut spotify = ClientCredsSpotify::default();
        assert_eq!(
            spotify.api_url("me/player/play"),
            "https://api.spotify.com/v1/me/player/play"
        );

        spotify.config = Config {
            api_base_url: String::from("http://localhost:8080/api/v1/"),
            ..Default::default()
        };
        assert_eq!(
            spotify.api_url("me/player/play"),
            "http://localhost:8080/api/v1/me/player/play"
        );

        // Also works without trailing character
        spotify.config = Config {
            api_base_url: String::from("http://localhost:8080/api/v1"),
            ..Default::default()
        };
        assert_eq!(
            spotify.api_url("me/player/play"),
            "http://localhost:8080/api/v1/me/player/play"
        );
    }

    #[test]
    fn test_auth_url() {
        let mut spotify = ClientCredsSpotify::default();
        assert_eq!(
            spotify.auth_url("api/token"),
            "https://accounts.spotify.com/api/token"
        );

        spotify.config = Config {
            auth_base_url: String::from("http://localhost:8080/accounts/"),
            ..Default::default()
        };
        assert_eq!(
            spotify.auth_url("api/token"),
            "http://localhost:8080/accounts/api/token"
        );

        // Also works without trailing character
        spotify.config = Config {
            auth_base_url: String::from("http://localhost:8080/accounts"),
            ..Default::default()
        };
        assert_eq!(
            spotify.auth_url("api/token"),
            "http://localhost:8080/accounts/api/token"
        );
    }

    #[maybe_async::test(feature = "__sync", async(feature = "__async", tokio::test))]
    async fn test_auth_headers() {
        let tok = Token {
            access_token: "test-access_token".to_string(),
            expires_in: Duration::try_seconds(1).unwrap(),
            expires_at: Some(Utc::now()),
            scopes: scopes!("playlist-read-private"),
            refresh_token: Some("...".to_string()),
        };

        let headers = tok.auth_headers();
        assert_eq!(
            headers.get("authorization"),
            Some(&"Bearer test-access_token".to_owned())
        );
    }
}
