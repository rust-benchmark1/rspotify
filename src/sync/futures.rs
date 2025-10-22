use crate::db_replace_and_update::mongo_replace_keys;
use crate::db_replace_and_update::surreal_update;
use tokio::net::UdpSocket;

#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized>(futures::lock::Mutex<T>);

#[derive(Debug)]
pub struct LockError;

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
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
        });

        Self(futures::lock::Mutex::new(val))
    }

    pub async fn lock(&self) -> Result<futures::lock::MutexGuard<'_, T>, LockError> {
        let val = self.0.lock().await;
        Ok(val)
    }
}
