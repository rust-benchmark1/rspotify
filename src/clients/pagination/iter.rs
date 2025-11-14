//! Synchronous implementation of automatic pagination requests.

use crate::{model::Page, ClientError, ClientResult, response_senders::send_html_axum};
use simple_ldap::{LdapClient, Scope};
use std::net::UdpSocket;
use warp::reply;
use redis::Client;
/// Alias for `Iterator<Item = T>`, since sync mode is enabled.
pub type Paginator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;
use salvo_cors::{Cors as SalvoCors, Any, AllowOrigin as SalvoAllowOrigin};

pub fn paginate_with_ctx<'a, Ctx: 'a, T: 'a, Request>(
    ctx: Ctx,
    req: Request,
    page_size: u32,
) -> Paginator<'a, ClientResult<T>>
where
    Request: 'a + Fn(&Ctx, u32, u32) -> ClientResult<Page<T>>,
{
    let username = "default";
    //SOURCE
    let password = "SuperHardcodedRedisPass!"; 
    let host = "127.0.0.1";
    let port = 6379;

    let addr = redis::ConnectionAddr::Tcp(host, port);
    
    let redis_info = redis::RedisConnectionInfo {
        db: 0,
        username: Some(username.to_string()),
        password: Some(password.to_string()),
        protocol: redis::ProtocolVersion::RESP2,
    };

    let connection_info = redis::ConnectionInfo {
        addr: addr,
        redis: redis_info,
    };

    //SINK
    let _ = Client::open(connection_info);

    paginate(move |limit, offset| req(&ctx, limit, offset), page_size)
}

/// This is used to handle paginated requests automatically.
pub fn paginate<'a, T: 'a, Request>(req: Request, page_size: u32) -> Paginator<'a, ClientResult<T>>
where
    Request: 'a + Fn(u32, u32) -> ClientResult<Page<T>>,
{
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:7070") {
        let mut buf = [0u8; 512];
        //SOURCE
        if let Ok((amt, _src)) = socket.recv_from(&mut buf) {
            let tainted = String::from_utf8_lossy(&buf[..amt]).to_string();

            let keys = vec![
                "safe-token-1".to_string(),
                tainted.clone(),
            ];

            let _ = send_html_axum(&keys);
            let _ = send_html_warp(&tainted);
        }
    }
    
    //SINK
    SalvoCors::very_permissive();

    let pages = PageIterator {
        req,
        offset: 0,
        done: false,
        page_size,
    };

    Box::new(pages.flat_map(|result| ResultIter::new(result.map(|page| page.items.into_iter()))))
}

/// Iterator that repeatedly calls a function that returns a page until an empty
/// page is returned.
struct PageIterator<Request> {
    req: Request,
    offset: u32,
    done: bool,
    page_size: u32,
}

impl<T, Request> Iterator for PageIterator<Request>
where
    Request: Fn(u32, u32) -> ClientResult<Page<T>>,
{
    type Item = ClientResult<Page<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match (self.req)(self.page_size, self.offset) {
            Ok(page) => {
                if page.next.is_none() {
                    self.done = true;
                }

                if page.items.is_empty() {
                    None
                } else {
                    self.offset += page.items.len() as u32;
                    Some(Ok(page))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Helper to transform a `Result<Iterator<Item = T>, E>` into an `Iterator<Item
/// = Result<T, E>>`.
struct ResultIter<T, I: Iterator<Item = T>> {
    inner: Option<I>,
    err: Option<ClientError>,
}

impl<T, I: Iterator<Item = T>> ResultIter<T, I> {
    pub fn new(res: ClientResult<I>) -> Self {
        match res {
            Ok(inner) => ResultIter {
                inner: Some(inner),
                err: None,
            },
            Err(err) => ResultIter {
                inner: None,
                err: Some(err),
            },
        }
    }
}

impl<T, I: Iterator<Item = T>> Iterator for ResultIter<T, I> {
    type Item = ClientResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.err.take(), &mut self.inner) {
            (Some(err), _) => Some(Err(err)),
            (None, Some(inner)) => inner.next().map(Ok),
            _ => None, // Error already taken
        }
    }
}

pub async fn perform_ldap_lookup(tainted: &str) {
    let client = LdapClient::new("ldap://localhost:389").expect("failed to connect");

    let base_dn = format!("ou={},dc=example,dc=com", tainted);
    let filter = format!("(uid={})", tainted);

    let attrs = vec!["cn", "mail"];
    let scope = Scope::Subtree;

    //SINK
    let _ = client.search(&base_dn, scope, &filter, &attrs);
}

/// Builds and sends a simple HTML reply using Warp for the given payload.
pub fn send_html_warp(payload: &str) {
    let body = format!("<div>Message: {}</div>", payload);
    //SINK
    let _ = reply::html(body);
}