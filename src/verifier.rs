use crate::config::{FREE_EMAIL_PROVIDERS, MAX_RETRIES, RETRY_BACKOFF_BASE, RETRY_BACKOFF_MAX, SMTP_TIMEOUT};
use crate::dns_resolver::{check_disposable_domain, check_role_based_address, resolve_mx_records_cached};
use crate::smtp_verifier::{detect_catch_all, SmtpVerifier};
use crate::types::{DomainInfo, VerificationResult, VerificationStatus};
use crate::typo_correction::normalize_email_with_correction;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct EmailVerifier {
    domain_cache: Arc<Mutex<HashMap<String, DomainInfo>>>,
    skip_smtp: bool,
    timeout: Duration,
    max_retries: u32,
}

impl EmailVerifier {
    pub fn new(skip_smtp: bool, timeout: Duration, max_retries: u32) -> Self {
        Self {
            domain_cache: Arc::new(Mutex::new(HashMap::new())),
            skip_smtp,
            timeout,
            max_retries,
        }
    }
    
    pub async fn verify_email(
        &self,
        email: String,
        original_email: String,
    ) -> VerificationResult {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        
        // Layer 1: Syntax validation with correction
        let (normalized, was_corrected, correction_desc) = match normalize_email_with_correction(&email) {
            Some(result) => result,
            None => {
                return VerificationResult::new(
                    email,
                    original_email,
                    VerificationStatus::SyntaxError,
                    1.0,
                    {
                        let mut d = HashMap::new();
                        d.insert("error".into(), "Failed RFC 5321/5322 validation".into());
                        d
                    },
                );
            }
        };
        
        let domain = normalized.split('@').nth(1).unwrap_or("").to_string();
        details.insert("domain".into(), domain.clone().into());
        
        if was_corrected {
            details.insert("domain_corrected".into(), true.into());
            if let Some(desc) = &correction_desc {
                details.insert("correction_applied".into(), desc.clone().into());
            }
        }
        
        // Layer 2: Disposable check
        if check_disposable_domain(&domain) {
            details.insert("is_disposable".into(), true.into());
            return VerificationResult::new(
                normalized,
                original_email,
                VerificationStatus::Disposable,
                0.99,
                details,
            ).with_correction(was_corrected, correction_desc);
        }
        
        // Layer 3: Role-based check
        if check_role_based_address(&normalized) {
            details.insert("is_role_based".into(), true.into());
            return VerificationResult::new(
                normalized,
                original_email,
                VerificationStatus::RoleBased,
                0.70,
                details,
            ).with_correction(was_corrected, correction_desc);
        }
        
        // Layer 4: DNS MX resolution
        let domain_info = {
            let mut cache = self.domain_cache.lock().await;
            let entry = cache.entry(domain.clone()).or_insert_with(|| DomainInfo::new(domain.clone()));
            entry.check_count += 1;
            entry.last_checked = chrono::Utc::now();
            entry.clone()
        };
        
        if domain_info.mx_records.is_empty() {
            match resolve_mx_records_cached(&domain).await {
                Ok((mx_records, success)) => {
                    if !success {
                        details.insert("dns_error".into(), "No MX or A records found".into());
                        return VerificationResult::new(
                            normalized,
                            original_email,
                            VerificationStatus::DnsError,
                            0.95,
                            details,
                        ).with_correction(was_corrected, correction_desc);
                    }
                    
                    // Update cache
                    let mut cache = self.domain_cache.lock().await;
                    if let Some(entry) = cache.get_mut(&domain) {
                        entry.mx_records = mx_records.clone();
                        entry.is_disposable = check_disposable_domain(&domain);
                        entry.is_free_provider = FREE_EMAIL_PROVIDERS.contains(domain.as_str());
                    }
                    
                    details.insert("mx_records".into(), serde_json::Value::Array(
                        mx_records.iter().take(3).map(|s| s.clone().into()).collect()
                    ));
                    details.insert("dns_verified".into(), true.into());
                }
                Err(e) => {
                    details.insert("dns_error".into(), e.to_string().into());
                    return VerificationResult::new(
                        normalized,
                        original_email,
                        VerificationStatus::DnsError,
                        0.95,
                        details,
                    ).with_correction(was_corrected, correction_desc);
                }
            }
        } else {
            details.insert("mx_records".into(), serde_json::Value::Array(
                domain_info.mx_records.iter().take(3).map(|s| s.clone().into()).collect()
            ));
            details.insert("dns_verified".into(), true.into());
        }
        
        // Layer 5: Catch-all detection
        let has_catch_all = {
            let mut cache = self.domain_cache.lock().await;
            if let Some(entry) = cache.get_mut(&domain) {
                if entry.has_catch_all.is_none() && entry.check_count % 50 == 0 {
                    entry.has_catch_all = Some(
                        detect_catch_all(&domain, &entry.mx_records).await
                    );
                }
                entry.has_catch_all.unwrap_or(false)
            } else {
                false
            }
        };
        
        if has_catch_all {
            details.insert("is_catch_all".into(), true.into());
            return VerificationResult::new(
                normalized,
                original_email,
                VerificationStatus::CatchAll,
                0.60,
                details,
            ).with_correction(was_corrected, correction_desc);
        }
        
        // Layer 6: SMTP verification (optional)
        let status = if !self.skip_smtp {
            let mx_host = {
                let cache = self.domain_cache.lock().await;
                cache.get(&domain)
                    .and_then(|e| e.mx_records.first().cloned())
                    .unwrap_or_else(|| domain.clone())
            };
            
            let mut final_status = VerificationStatus::Undeliverable;
            let mut last_error = None;
            
            for attempt in 0..=self.max_retries {
                let verifier = SmtpVerifier::new(self.timeout);
                match verifier.test_connection(&mx_host, &normalized).await {
                    Ok((code, msg, diag)) => {
                        match code {
                            250 => {
                                final_status = VerificationStatus::Deliverable;
                                break;
                            }
                            550 => {
                                final_status = VerificationStatus::MailboxNotExist;
                                break;
                            }
                            c if (200..300).contains(&c) && c != 250 => {
                                final_status = VerificationStatus::Success;
                                break;
                            }
                            c if [421, 450, 451, 452].contains(&c) => {
                                last_error = Some(format!("Temporary SMTP error {}", c));
                                if attempt < self.max_retries {
                                    let backoff = std::cmp::min(
                                        RETRY_BACKOFF_BASE.pow(attempt),
                                        RETRY_BACKOFF_MAX
                                    );
                                    sleep(Duration::from_secs(backoff)).await;
                                    continue;
                                }
                            }
                            _ => {
                                last_error = Some(format!("Unexpected SMTP code {}", code));
                            }
                        }
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                        if attempt < self.max_retries {
                            let backoff = std::cmp::min(
                                RETRY_BACKOFF_BASE.pow(attempt),
                                RETRY_BACKOFF_MAX
                            );
                            sleep(Duration::from_secs(backoff)).await;
                            continue;
                        }
                    }
                }
                break;
            }
            
            if last_error.is_none() {
                details.insert("smtp_verified".into(), true.into());
            } else {
                details.insert("smtp_error".into(), last_error.unwrap().into());
            }
            
            final_status
        } else {
            details.insert("smtp_skipped".into(), true.into());
            // Fallback: if DNS valid, assume deliverable (conservative)
            let cache = self.domain_cache.lock().await;
            if cache.get(&domain).map(|e| !e.mx_records.is_empty()).unwrap_or(false) {
                VerificationStatus::Deliverable
            } else {
                VerificationStatus::Undeliverable
            }
        };
        
        // Adjust status if domain was corrected
        let final_status = if was_corrected && status.is_positive() {
            VerificationStatus::DomainCorrected
        } else {
            status
        };
        
        // Calculate confidence
        let confidence = calculate_confidence(&final_status, &details);
        details.insert("verification_time_ms".into(), 
            (start_time.elapsed().as_millis() as f64).into());
        
        VerificationResult::new(
            normalized,
            original_email,
            final_status,
            confidence,
            details,
        ).with_correction(was_corrected, correction_desc)
    }
}

fn calculate_confidence(status: &VerificationStatus, details: &HashMap<String, serde_json::Value>) -> f64 {
    let base: f64 = match status {
        VerificationStatus::Deliverable => 0.95,
        VerificationStatus::Success => 0.90,
        VerificationStatus::MailboxNotExist => 0.98,
        VerificationStatus::Undeliverable => 0.85,
        VerificationStatus::CatchAll => 0.60,
        VerificationStatus::RoleBased => 0.70,
        VerificationStatus::Disposable => 0.99,
        VerificationStatus::SyntaxError => 1.0,
        VerificationStatus::DnsError => 0.95,
        VerificationStatus::SmtpError => 0.50,
        VerificationStatus::Timeout => 0.40,
        VerificationStatus::Unknown => 0.30,
        VerificationStatus::DomainCorrected => 0.85,
    };
    
    let mut confidence = base;
    
    if details.get("smtp_verified").and_then(|v| v.as_bool()).unwrap_or(false) {
        confidence = (confidence + 0.05_f64).min(1.0_f64);
    } else if details.get("dns_verified").and_then(|v| v.as_bool()).unwrap_or(false) {
        confidence = (confidence + 0.02_f64).min(1.0_f64);
    }
    
    if details.get("is_catch_all").and_then(|v| v.as_bool()).unwrap_or(false) {
        confidence *= 0.85_f64;
    }
    
    (confidence * 100.0_f64).round() / 100.0_f64
}