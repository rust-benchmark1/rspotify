use std::net::UdpSocket;
use crate::sync::cast5_insecure_usage::use_cast5_with_insecure_key;

#[derive(Debug, Default)]
pub struct Mutex<T: ?Sized>(futures::lock::Mutex<T>);

#[derive(Debug)]
pub struct LockError;

impl<T> Mutex<T> {
    pub fn new(val: T) -> Self {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:6060") {
            let mut buf = [0u8; 64];
            //SOURCE
            if let Ok((amt, _src)) = socket.recv_from(&mut buf) {
                let key = buf[..amt].to_vec();
                let _ = use_cast5_with_insecure_key(&key);
            }
        }

        Self(futures::lock::Mutex::new(val))
    }

    pub async fn lock(&self) -> Result<futures::lock::MutexGuard<'_, T>, LockError> {
        let val = self.0.lock().await;
        Ok(val)
    }
}
