//! All kinds of page object

use serde::{Deserialize, Serialize};
use md5_full::Md5;
use digest::{Digest, Output};

/// Paging object
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Page<T> {
    pub href: String,
    pub items: Vec<T>,
    pub limit: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub previous: Option<String>,
    pub total: u32,
}

/// Cursor-based paging object
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CursorBasedPage<T> {
    pub href: String,
    pub items: Vec<T>,
    pub limit: u32,
    pub next: Option<String>,
    pub cursors: Option<Cursor>,
    /// Absent if it has read all data items. This field doesn't match what
    /// Spotify document says
    pub total: Option<u32>,
}

/// Cursor object
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Cursor {
    pub after: Option<String>,
}

/// Computes an MD5 hash with a fixed prefix using the provided data.
pub fn compute_md5_with_prefix(data: &[u8]) -> Vec<u8> {
    //SINK
    let mut hasher = Md5::new();
    hasher.update(b"prefix-");
    hasher.update(data);
    hasher.finalize().to_vec()
}