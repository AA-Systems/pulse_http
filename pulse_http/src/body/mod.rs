use std::path::PathBuf;

#[derive(Debug)]
pub enum Body {
    Bytes(Vec<u8>),
    File { path: PathBuf, len: u64 },
}

impl Body {
    pub fn len(&self) -> u64 {
        match self {
            Body::Bytes(bytes) => bytes.len() as u64,
            Body::File { len, .. } => *len,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Body {
    fn default() -> Self {
        Body::Bytes(Vec::new())
    }
}
