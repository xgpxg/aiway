use reqwest::{Client, ClientBuilder};

pub struct HttpLogWriter {
    client: Client,
    endpoint: String,
}

impl HttpLogWriter {
    pub fn new<E: Into<String>>(endpoint: E) -> Self {
        Self {
            client: ClientBuilder::default()
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .unwrap(),
            endpoint: endpoint.into(),
        }
    }
}

impl std::io::Write for HttpLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = buf.to_vec();
        let endpoint = self.endpoint.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            if let Err(e) = client.post(&endpoint).body(message).send().await {
                println!("write log to http endpoint error: {}", e);
            }
        });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
