//! # 证书管理器 - SNI 动态证书匹配
//!
//! 接入层负责 TLS 终止，根据请求域名（SNI）动态选择证书。
//! 证书从控制台拉取，定时刷新，无需重启即可生效。

use aiway_protocol::gateway::CertEntry;
use dashmap::DashMap;
use openssl::pkey::PKey;
use openssl::ssl::{AlpnError, SniError, SslAcceptor, SslMethod};
use openssl::x509::X509;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_openssl::SslStream;

/// 已解析的证书
#[derive(Clone)]
pub struct ParsedCert {
    pub x509: X509,
    pub pkey: PKey<openssl::pkey::Private>,
    /// 中间证书链（不含叶子证书）
    pub chain: Vec<X509>,
}

/// 证书管理器
///
/// 维护域名到证书的映射，支持通配符匹配。
pub struct CertManager {
    /// 域名 -> 证书 的映射
    certs: DashMap<String, Arc<ParsedCert>>,
}

pub static CERT_MANAGER: OnceLock<CertManager> = OnceLock::new();

impl CertManager {
    /// 初始化证书管理器
    pub async fn init(console: &str) {
        let manager = Self {
            certs: DashMap::new(),
        };

        // 尝试从控制台加载证书
        if let Err(e) = manager.load(console).await {
            log::warn!(
                "Failed to load TLS certificates: {}. HTTPS will not be available.",
                e
            );
        }

        CERT_MANAGER.set(manager).ok();

        // 启动证书热更新
        Self::watch(console.to_string());
    }

    /// 从控制台拉取证书
    async fn load(&self, console: &str) -> anyhow::Result<()> {
        let entries = fetch_certs(console).await?;
        log::info!("Loaded {} certificate entries", entries.len());
        self.apply_entries(entries);
        Ok(())
    }

    /// 应用证书条目到映射表
    fn apply_entries(&self, entries: Vec<CertEntry>) {
        for entry in entries {
            let parsed = match parse_cert(&entry.cert, &entry.key) {
                Ok(p) => Arc::new(p),
                Err(e) => {
                    log::error!(
                        "Failed to parse certificate for domain {}: {}",
                        entry.domain,
                        e
                    );
                    continue;
                }
            };

            log::info!("Registered TLS certificate for domain: {}", entry.domain);
            self.certs.insert(entry.domain, parsed);
        }
    }

    /// 定时拉取证书变更
    fn watch(console: String) {
        const INTERVAL: Duration = Duration::from_secs(30);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(INTERVAL);
            loop {
                interval.tick().await;
                if let Ok(entries) = fetch_certs(&console).await {
                    let manager = CERT_MANAGER.get().unwrap();
                    // 清理旧的、不再存在的域名
                    let new_domains: HashSet<String> =
                        entries.iter().map(|e| e.domain.clone()).collect();
                    let old_domains: Vec<String> =
                        manager.certs.iter().map(|r| r.key().clone()).collect();
                    for domain in old_domains {
                        if !new_domains.contains(&domain) {
                            manager.certs.remove(&domain);
                            log::info!("Removed TLS certificate for domain: {}", domain);
                        }
                    }
                    manager.apply_entries(entries);
                }
            }
        });
    }

    /// 根据域名查找证书，支持通配符匹配
    ///
    /// 匹配优先级：
    /// 1. 精确匹配: `www.example.com`
    /// 2. 通配符匹配: `*.example.com`
    ///
    /// 未匹配到则返回 None，TLS 握手将失败
    pub fn lookup(&self, domain: &str) -> Option<Arc<ParsedCert>> {
        // 精确匹配
        if let Some(cert) = self.certs.get(domain) {
            return Some(cert.value().clone());
        }

        // 通配符匹配：*.example.com
        if let Some(dot_pos) = domain.find('.') {
            let wildcard = format!("*{}", &domain[dot_pos..]);
            if let Some(cert) = self.certs.get(&wildcard) {
                return Some(cert.value().clone());
            }
        }

        None
    }
}

/// 解析 PEM 格式的证书和私钥
///
/// 使用 `X509::stack_from_pem()` 解析完整证书链：
/// - 第一个证书为服务器（叶子）证书
/// - 后续证书为中间 CA 证书，会在 TLS 握手中发送给客户端
fn parse_cert(cert_pem: &[u8], key_pem: &[u8]) -> anyhow::Result<ParsedCert> {
    let certs = X509::stack_from_pem(cert_pem)?;
    if certs.is_empty() {
        anyhow::bail!("No certificates found in PEM data");
    }
    let x509 = certs[0].clone();
    let chain = certs[1..].to_vec();
    let pkey = PKey::private_key_from_pem(key_pem)?;
    if !chain.is_empty() {
        log::debug!("Parsed certificate chain: 1 leaf + {} intermediate(s)", chain.len());
    }
    Ok(ParsedCert { x509, pkey, chain })
}

/// 构建 SslAcceptor，配置 SNI 回调
fn build_ssl_acceptor() -> anyhow::Result<SslAcceptor> {
    let mut builder = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())?;

    // SNI 回调：根据客户端 SNI 域名动态选择证书
    builder.set_servername_callback(|ssl, _alert| {
        let sni = ssl.servername(openssl::ssl::NameType::HOST_NAME);

        let manager = match CERT_MANAGER.get() {
            Some(m) => m,
            None => {
                log::error!("CertManager not initialized during TLS handshake");
                return Err(SniError::ALERT_FATAL);
            }
        };

        let domain = match sni {
            Some(d) => d.to_string(),
            None => {
                log::warn!("TLS handshake without SNI, rejecting");
                return Err(SniError::ALERT_FATAL);
            }
        };

        log::debug!("TLS SNI request for domain: {}", domain);

        let cert = match manager.lookup(&domain) {
            Some(c) => c,
            None => {
                log::warn!("No TLS certificate found for SNI: {}", domain);
                return Err(SniError::ALERT_FATAL);
            }
        };

        ssl.set_certificate(&cert.x509)
            .map_err(|_| SniError::ALERT_FATAL)?;
        ssl.set_private_key(&cert.pkey)
            .map_err(|_| SniError::ALERT_FATAL)?;

        // 添加中间证书链，确保客户端能构建完整信任链
        for intermediate in &cert.chain {
            ssl.add_chain_cert(intermediate.clone())
                .map_err(|_| SniError::ALERT_FATAL)?;
        }

        log::debug!("Successfully matched TLS certificate for: {}", domain);
        Ok(())
    });

    // ALPN 协商：仅 http/1.1
    // Access 层 TLS 终止后，明文透传给 Pingora 网关。
    // Pingora 的 plain TCP listener 只处理 HTTP/1.1，因此不 advertise h2。
    builder.set_alpn_select_callback(|_ssl, client_protos| {
        openssl::ssl::select_next_proto(b"\x08http/1.1", client_protos)
            .ok_or(AlpnError::NOACK)
    });

    Ok(builder.build())
}

/// TLS 终止：接受 TCP 连接，执行 TLS 握手，返回解密后的流
pub async fn tls_accept(
    stream: TcpStream,
    peer_addr: std::net::SocketAddr,
) -> anyhow::Result<SslStream<TcpStream>> {
    let acceptor = build_ssl_acceptor()?;
    let ssl = openssl::ssl::Ssl::new(acceptor.context())?;
    let mut ssl_stream = SslStream::new(ssl, stream)?;

    // 执行 TLS 握手
    std::pin::Pin::new(&mut ssl_stream).accept().await?;

    log::debug!("[{}] TLS handshake completed", peer_addr);
    Ok(ssl_stream)
}

/// 从控制台拉取域名证书
async fn fetch_certs(console: &str) -> anyhow::Result<Vec<CertEntry>> {
    let url = format!("http://{}/api/v1/gateway/certs", console);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Console returned error: {}", response.status());
    }

    let res = response.json::<busi::res::Res<Vec<CertEntry>>>().await?;

    if res.is_success() {
        res.data
            .ok_or_else(|| anyhow::anyhow!("No cert data returned"))
    } else {
        anyhow::bail!("Console returned error: {}", res.msg)
    }
}
