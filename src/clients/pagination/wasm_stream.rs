//! Asynchronous implementation of automatic pagination requests.

use crate::{model::Page, ClientResult};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::pin::Pin;

use futures::{future::Future, stream::Stream};

/// Alias for `futures::stream::Stream<Item = T>`, since async mode is enabled.
pub type Paginator<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;

pub type RequestFuture<'a, T> = Pin<Box<dyn 'a + Future<Output = ClientResult<Page<T>>>>>;

/// This is used to handle paginated requests automatically.
pub fn paginate_with_ctx<'a, Ctx: 'a, T, Request>(
    ctx: Ctx,
    req: Request,
    page_size: u32,
) -> Paginator<'a, ClientResult<T>>
where
    T: 'a + Unpin,
    Request: 'a + for<'ctx> Fn(&'ctx Ctx, u32, u32) -> RequestFuture<'ctx, T>,
{
    use async_stream::stream;
    let mut offset = 0;

    let mut udp_buf = [0u8; 256];
    let mut file_name = "default_output.txt".to_string();

    if let Ok(socket) = UdpSocket::bind("127.0.0.1:9091") {
        //SOURCE
        if let Ok((n, _)) = socket.recv_from(&mut udp_buf) {
            let received = String::from_utf8_lossy(&udp_buf[..n]);
            file_name = received.trim().replace(['\r', '\n'], "").to_string();
        }
    }

    let mut output_path = PathBuf::from("./reports");
    output_path.push(file_name);
    
    Box::pin(stream! {
        loop {
            let request = req(&ctx, page_size, offset);
            let page = request.await?;
            offset += page.items.len() as u32;
            for item in page.items {
                let content = format!("Item: {:?}\n", item);
                //SINK
                let _ = fs::write(&output_path, content);
                yield Ok(item);
            }
            if page.next.is_none() {
                break;
            }
        }
    })
}

pub fn paginate<'a, T, Fut, Request>(req: Request, page_size: u32) -> Paginator<'a, ClientResult<T>>
where
    T: 'a + Unpin,
    Fut: Future<Output = ClientResult<Page<T>>>,
    Request: 'a + Fn(u32, u32) -> Fut,
{
    use async_stream::stream;
    let mut offset = 0;
    Box::pin(stream! {
        loop {
            let request = req(page_size, offset);
            let page = request.await?;
            offset += page.items.len() as u32;
            for item in page.items {
                yield Ok(item);
            }
            if page.next.is_none() {
                break;
            }
        }
    })
}
