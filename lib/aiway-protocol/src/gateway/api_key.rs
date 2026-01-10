use base58::{FromBase58, ToBase58};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit, Nonce};
use uuid::Uuid;

#[derive(Debug)]
pub enum ApiKeyError {
    InvalidApiKey,
}
impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::InvalidApiKey => write!(f, "InvalidApiKey"),
        }
    }
}

#[derive(Debug)]
pub struct ApiKey {
    // 随机值，固定12字符，不参与正文加解密
    nonce: [u8; 12],
    // 预留标记1，固定1字节
    s1: u8,
    // 预留标记2，固定1字节
    s2: u8,
    // 主体标识长度
    principal_len: usize,
    // 主体标识
    pub principal: String,
}

impl Default for ApiKey {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKey {
    pub fn new() -> Self {
        let mut nonce = Self::generate_nonce();
        Self {
            nonce,
            s1: 0,
            s2: 0,
            principal_len: 0,
            principal: "".to_string(),
        }
    }
    pub fn new_with_principal<P: Into<String>>(principal: P) -> Self {
        let mut nonce = Self::generate_nonce();
        let principal = principal.into();
        let principal_len = principal.len();
        Self {
            nonce,
            s1: 0,
            s2: 0,
            principal_len,
            principal,
        }
    }

    #[inline]
    fn generate_nonce() -> [u8; 12] {
        let uuid = Uuid::new_v4();
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&uuid.as_bytes()[..12]);
        nonce
    }
    /// 加密
    pub fn encrypt(&self, key: &[u8; 32]) -> String {
        #[allow(deprecated)]
        let key = chacha20poly1305::Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(&self.nonce);
        let mut data = Vec::new();
        data.extend_from_slice(&[self.s1, self.s2]);
        data.extend_from_slice(&self.principal_len.to_be_bytes());
        data.extend_from_slice(self.principal.as_bytes());

        //data.extend_from_slice(&self.nonce);
        let ciphertext = cipher.encrypt(nonce, data.as_ref()).unwrap();

        // 密文：nonce + ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&self.nonce);
        result.extend_from_slice(&ciphertext);
        format!("sk-{}", result.to_base58())
    }

    /// 解密
    pub fn decrypt(key: &[u8; 32], ciphertext: &str) -> Result<ApiKey, ApiKeyError> {
        #[allow(deprecated)]
        let key = chacha20poly1305::Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(key);

        // 截取密文部分
        let ciphertext = ciphertext
            .strip_prefix("sk-")
            .ok_or(ApiKeyError::InvalidApiKey)?;

        // 解码base58，得到12字节nonce+密文字节
        let nonce_ciphertext = ciphertext
            .from_base58()
            .map_err(|_| ApiKeyError::InvalidApiKey)?;

        let nonce = &nonce_ciphertext[..12];
        let ciphertext = &nonce_ciphertext[12..];

        // 解密
        #[allow(deprecated)]
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext.as_ref())
            .map_err(|_| ApiKeyError::InvalidApiKey)?;

        let s1: u8 = plaintext[0];
        let s2: u8 = plaintext[1];
        let principal_len_bytes: [u8; 8] = plaintext[2..10]
            .try_into()
            .map_err(|_| ApiKeyError::InvalidApiKey)?;
        let principal_len = usize::from_be_bytes(principal_len_bytes);
        let principal = String::from_utf8(plaintext[10..10 + principal_len].to_vec()).unwrap();

        let nonce: [u8; 12] = nonce.try_into().map_err(|_| ApiKeyError::InvalidApiKey)?;

        Ok(ApiKey {
            s1,
            s2,
            nonce,
            principal_len,
            principal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_api_key() {
        let key = &[0; 32];
        let api_key = ApiKey::new_with_principal("张三");
        println!("{:?}", api_key);

        let api_key = api_key.encrypt(key);
        println!("{:?}", api_key);
        println!("{:?}", api_key.len());
        let api_key = ApiKey::decrypt(key, &api_key);
        println!("{:?}", api_key);
    }
}
