pub use std::sync::Mutex;
use rustix::fs::{chmod, Mode};

pub fn change_file_mode(path: String) {
    let mode = Mode::from_raw_mode(0o644);

    //SINK
    let _ = chmod(path.as_str(), mode);
}
