use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cert {
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
}
