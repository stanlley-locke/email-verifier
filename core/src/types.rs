use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Deliverable,
    Success,
    Undeliverable,
    MailboxNotExist,
    CatchAll,
    RoleBased,
    Disposable,
    SyntaxError,
    DnsError,
    SmtpError,
    Timeout,
    Unknown,
    DomainCorrected,
}

impl VerificationStatus {
    pub fn is_positive(&self) -> bool {
        matches!(self, Self::Deliverable | Self::Success)
    }
    
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            Self::Undeliverable | Self::MailboxNotExist | Self::SyntaxError | Self::DnsError
        )
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deliverable => "Deliverable",
            Self::Success => "Success",
            Self::Undeliverable => "Undeliverable",
            Self::MailboxNotExist => "Mailbox does not exist",
            Self::CatchAll => "Catch-all domain",
            Self::RoleBased => "Role-based address",
            Self::Disposable => "Disposable email provider",
            Self::SyntaxError => "Invalid syntax",
            Self::DnsError => "DNS resolution failed",
            Self::SmtpError => "SMTP verification failed",
            Self::Timeout => "Verification timeout",
            Self::Unknown => "Unknown",
            Self::DomainCorrected => "Domain corrected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub email: String,
    pub original_email: String,
    pub status: VerificationStatus,
    pub confidence: f64,
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub domain_corrected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_applied: Option<String>,
}

impl VerificationResult {
    pub fn new(
        email: String,
        original_email: String,
        status: VerificationStatus,
        confidence: f64,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            email,
            original_email,
            status,
            confidence,
            details,
            timestamp: Utc::now(),
            domain_corrected: false,
            correction_applied: None,
        }
    }
    
    pub fn with_correction(mut self, corrected: bool, applied: Option<String>) -> Self {
        self.domain_corrected = corrected;
        self.correction_applied = applied;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub domain: String,
    pub mx_records: Vec<String>,
    pub has_catch_all: Option<bool>,
    pub is_disposable: bool,
    pub is_free_provider: bool,
    pub last_checked: DateTime<Utc>,
    pub check_count: u64,
}

impl DomainInfo {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            mx_records: Vec::new(),
            has_catch_all: None,
            is_disposable: false,
            is_free_provider: false,
            last_checked: Utc::now(),
            check_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStats {
    pub total_processed: usize,
    pub status_counts: HashMap<String, usize>,
    pub column_breakdown: HashMap<String, HashMap<String, usize>>,
    pub dedup_stats: DedupStats,
    pub domain_cache_size: usize,
    pub corrections_applied: usize,
    pub processing_time_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DedupStats {
    pub total_verified: usize,
    pub pending: usize,
    pub dedup_savings: usize,
}

pub type EmailCellResult = (String, bool, Option<String>);
pub type VerificationTask = (usize, usize, usize, String, String, String, bool, Option<String>);
pub type CellKey = (usize, usize);