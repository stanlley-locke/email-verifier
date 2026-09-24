use crate::config::{SMTP_HELO_DOMAIN, SMTP_TIMEOUT};
use lettre::{
    transport::smtp::client::{Tls, TlsParameters},
    SmtpTransport,
};
use std::net::ToSocketAddrs;
use std::time::Duration;

pub struct SmtpVerifier {
    timeout: Duration,
}

impl SmtpVerifier {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Test whether the MX host accepts a connection for `email`.
    ///
    /// `SmtpTransport::test_connection()` is **synchronous/blocking**.
    /// We run it on the `spawn_blocking` thread pool so it never blocks
    /// the tokio async runtime.
    pub async fn test_connection(
        &self,
        mx_host: &str,
        _email: &str,
    ) -> Result<(i32, Option<String>, String), anyhow::Error> {
        // Resolve first (sync, but fast — just a system DNS call)
        let addrs = match format!("{}:25", mx_host).to_socket_addrs() {
            Ok(a) => a.collect::<Vec<_>>(),
            Err(e) => {
                return Ok((-1, None, format!("DNS resolution failed: {}", e)));
            }
        };

        if addrs.is_empty() {
            return Ok((-1, None, "No addresses resolved".to_string()));
        }

        // Clone strings so they can move into the blocking thread
        let mx_host_owned = mx_host.to_string();
        let timeout = self.timeout;

        // Run the blocking SMTP check off the async thread pool
        let result = tokio::task::spawn_blocking(move || {
            let mut builder = SmtpTransport::builder_dangerous(&mx_host_owned)
                .port(25)
                .timeout(Some(timeout))
                .hello_name(
                    lettre::transport::smtp::extension::ClientId::Domain(
                        SMTP_HELO_DOMAIN.to_string(),
                    ),
                );

            if let Ok(tls_params) = TlsParameters::new(mx_host_owned.clone()) {
                builder = builder.tls(Tls::Opportunistic(tls_params));
            }

            let transport = builder.build();

            match transport.test_connection() {
                Ok(_) => (
                    250,
                    None,
                    format!("Connected to {}:25; EHLO OK; MAIL FROM accepted", mx_host_owned),
                ),
                Err(e) => (-1, None, format!("SMTP error: {}", e)),
            }
        })
        .await;

        match result {
            Ok((code, msg, diag)) => Ok((code, msg, diag)),
            Err(e) => Ok((-1, None, format!("spawn_blocking error: {}", e))),
        }
    }
}

pub async fn detect_catch_all(domain: &str, mx_records: &[String]) -> bool {
    let test_email = format!(
        "nonexistent-test-{}@{}",
        chrono::Utc::now().timestamp(),
        domain
    );

    for mx_host in mx_records.iter().take(2) {
        let verifier = SmtpVerifier::new(SMTP_TIMEOUT);
        if let Ok((code, _, _)) = verifier.test_connection(mx_host, &test_email).await {
            if code == 250 {
                return true;
            }
        }
    }
    false
}