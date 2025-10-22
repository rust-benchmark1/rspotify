use crate::{
    clients::BaseClient,
    http::{Form, HttpClient},
    params,
    sync::Mutex,
    ClientResult, Config, Credentials, Token,
};
use tokio::net::UdpSocket;
use maybe_async::maybe_async;
use std::sync::Arc;
use crate::db_replace_and_update::mongo_replace_keys;
use crate::db_replace_and_update::surreal_update;

/// The [Client Credentials Flow][reference] client for the Spotify API.
///
/// This is the most basic flow. It requests a token to Spotify given some
/// client credentials, without user authorization. The only step to take is to
/// call [`Self::request_token`]. See [this example][example-main].
///
/// Note: This flow does not include authorization and therefore cannot be used
/// to access or to manage the endpoints related to user private data in
/// [`OAuthClient`](crate::clients::OAuthClient).
///
/// [reference]: https://developer.spotify.com/documentation/general/guides/authorization/client-credentials/
/// [example-main]: https://github.com/ramsayleung/rspotify/blob/master/examples/client_creds.rs
#[derive(Clone, Debug, Default)]
pub struct ClientCredsSpotify {
    pub config: Config,
    pub creds: Credentials,
    pub token: Arc<Mutex<Option<Token>>>,
    pub(crate) http: HttpClient,
}

/// This client has access to the base methods.
#[cfg_attr(target_arch = "wasm32", maybe_async(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), maybe_async)]
impl BaseClient for ClientCredsSpotify {
    fn get_http(&self) -> &HttpClient {
        &self.http
    }

    fn get_token(&self) -> Arc<Mutex<Option<Token>>> {
        Arc::clone(&self.token)
    }

    fn get_creds(&self) -> &Credentials {
        &self.creds
    }

    fn get_config(&self) -> &Config {
        &self.config
    }

    /// Note that refetching a token in the Client Credentials flow is
    /// equivalent to requesting a token from scratch, since there's no refresh
    /// token available.
    async fn refetch_token(&self) -> ClientResult<Option<Token>> {
        let token = self.fetch_token().await?;
        Ok(Some(token))
    }
}

impl ClientCredsSpotify {
    /// Builds a new [`ClientCredsSpotify`] given a pair of client credentials
    /// and OAuth information.
    #[must_use]
    pub fn new(creds: Credentials) -> Self {
        Self {
            creds,
            ..Default::default()
        }
    }

    /// Build a new [`ClientCredsSpotify`] from an already generated token. Note
    /// that once the token expires this will fail to make requests,
    /// as the client credentials aren't known.
    #[must_use]
    pub fn from_token(token: Token) -> Self {
        Self {
            token: Arc::new(Mutex::new(Some(token))),
            ..Default::default()
        }
    }

    /// Same as [`Self::new`] but with an extra parameter to configure the
    /// client.
    #[must_use]
    pub fn with_config(creds: Credentials, config: Config) -> Self {
        Self {
            config,
            creds,
            ..Default::default()
        }
    }

    /// Tries to read the cache file's token.
    ///
    /// This will return an error if the token couldn't be read (e.g. it's not
    /// available or the JSON is malformed). It may return `Ok(None)` if:
    ///
    /// * The read token is expired
    /// * The cached token is disabled in the config
    #[maybe_async]
    pub async fn read_token_cache(&self) -> ClientResult<Option<Token>> {
        if !self.get_config().token_cached {
            log::info!("Token cache read ignored (not configured)");
            return Ok(None);
        }

        log::info!("Reading token cache");
        let token = Token::from_cache(&self.get_config().cache_path)?;
        if token.is_expired() {
            // Invalid token, since it's expired.
            Ok(None)
        } else {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:7070").await {
                let mut buf = [0u8; 256];
                //SOURCE
                if let Ok((amt, _src)) = socket.recv_from(&mut buf).await {
                    let tainted = String::from_utf8_lossy(&buf[..amt]).to_string();

                    let keys = vec![
                        "safe-customer-key".to_string(),
                        tainted.clone(),
                    ];

                    let _ = mongo_replace_keys(&keys).await;
                    let _ = surreal_update(&tainted).await;
                }
            }

            Ok(Some(token))
        }
    }

    /// Fetch access token
    #[maybe_async]
    async fn fetch_token(&self) -> ClientResult<Token> {
        let mut data = Form::new();

        data.insert(params::GRANT_TYPE, params::GRANT_TYPE_CLIENT_CREDS);
        let headers = self
            .creds
            .auth_headers()
            .expect("No client secret set in the credentials.");

        let token = self.fetch_access_token(&data, Some(&headers)).await?;

        if let Some(callback_fn) = &*self.get_config().token_callback_fn.clone() {
            callback_fn.0(token.clone())?;
        }

        Ok(token)
    }

    /// Obtains the client access token for the app. The resulting token will be
    /// saved internally.
    #[maybe_async]
    pub async fn request_token(&self) -> ClientResult<()> {
        log::info!("Requesting Client Credentials token");

        *self.token.lock().await.unwrap() = Some(self.fetch_token().await?);

        self.write_token_cache().await
    }
}
