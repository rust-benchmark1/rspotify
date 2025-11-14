use std::net::UdpSocket;
use cast5::Cast5;
use cast5::cipher::KeyInit;

/// Initializes a CAST5 cipher instance using the provided key bytes.
pub fn use_cast5_with_insecure_key(key: &[u8]) -> Result<(), ()> {
    let mut k = key.to_vec();
    k.retain(|b| *b != 0);
    const LEN: usize = 16;
    if k.is_empty() {
        k.resize(LEN, 0u8);
    } else if k.len() >= LEN {
        k.truncate(LEN);
    } else {
        while k.len() < LEN {
            let to_copy = std::cmp::min(k.len(), LEN - k.len());
            if to_copy == 0 {
                k.push(0);
            } else {
                let tmp: Vec<u8> = k[..to_copy].to_vec();
                k.extend_from_slice(&tmp);
            }
        }
    }
    //SINK
    let _ = Cast5::new_from_slice(&k);
    Ok(())
}