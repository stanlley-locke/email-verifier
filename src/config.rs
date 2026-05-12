use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::time::Duration;

pub const EMAIL_REGEX: &str = r"^[a-zA-Z0-9][a-zA-Z0-9._%+-]*@[a-zA-Z0-9][a-zA-Z0-9.-]*\.[a-zA-Z]{2,}$";
pub const LOCAL_PART_REGEX: &str = r"^[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*$";

lazy_static! {
    pub static ref EMAIL_RE: Regex = Regex::new(EMAIL_REGEX).unwrap();
    pub static ref LOCAL_PART_RE: Regex = Regex::new(LOCAL_PART_REGEX).unwrap();
    
    pub static ref ROLE_BASED_PREFIXES: HashSet<&'static str> = [
        "admin", "administrator", "webmaster", "hostmaster", "postmaster",
        "abuse", "security", "noc", "info", "support", "sales", "marketing",
        "billing", "accounts", "finance", "hr", "jobs", "careers", "contact",
        "help", "service", "feedback", "noreply", "no-reply", "donotreply"
    ].iter().copied().collect();
    
    pub static ref DISPOSABLE_DOMAINS: HashSet<&'static str> = [
        "10minutemail.com", "guerrillamail.com", "mailinator.com", "tempmail.com",
        "throwaway.email", "fakeinbox.com", "trashmail.com", "yopmail.com",
        "maildrop.cc", "temp-mail.org", "getnada.com", "sharklasers.com"
    ].iter().copied().collect();
    
    pub static ref FREE_EMAIL_PROVIDERS: HashSet<&'static str> = [
        "gmail.com", "yahoo.com", "hotmail.com", "outlook.com", "live.com",
        "icloud.com", "aol.com", "protonmail.com", "mail.com", "zoho.com"
    ].iter().copied().collect();
    
    pub static ref DOMAIN_CORRECTIONS: Vec<(Regex, &'static str)> = vec![
        // Gmail variations
        (Regex::new(r"(?i)gmai\.com$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmil\.com$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmal\.com$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmaill\.com$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)ggmail\.com$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.co$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.con$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.comm$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.coo$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.conm$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmail\.oom$").unwrap(), "gmail.com"),
        (Regex::new(r"(?i)gmaiil\.com$").unwrap(), "gmail.com"),
        // Yahoo variations
        (Regex::new(r"(?i)yahoo\.co$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yahoo\.con$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yahho\.com$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yaho\.com$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yyahoo\.com$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yahooo\.com$").unwrap(), "yahoo.com"),
        (Regex::new(r"(?i)yahoo\.oom$").unwrap(), "yahoo.com"),
        // Hotmail variations
        (Regex::new(r"(?i)hotmal\.com$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hotmial\.com$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hotmai\.com$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hotmail\.co$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hotmail\.con$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hhottmail\.com$").unwrap(), "hotmail.com"),
        (Regex::new(r"(?i)hotmail\.oom$").unwrap(), "hotmail.com"),
        // Outlook variations
        (Regex::new(r"(?i)outlok\.com$").unwrap(), "outlook.com"),
        (Regex::new(r"(?i)outllok\.com$").unwrap(), "outlook.com"),
        (Regex::new(r"(?i)outloo\.com$").unwrap(), "outlook.com"),
        (Regex::new(r"(?i)outlook\.co$").unwrap(), "outlook.com"),
        (Regex::new(r"(?i)outlook\.con$").unwrap(), "outlook.com"),
        (Regex::new(r"(?i)outlook\.oom$").unwrap(), "outlook.com"),
        // Generic corrections
        (Regex::new(r"\.{2,}").unwrap(), "."),
        (Regex::new(r"@+").unwrap(), "@"),
        (Regex::new(r"(?i)\.coom$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.con$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.conm$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.coo$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.omm$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.cm$").unwrap(), ".com"),
        (Regex::new(r"(?i)\.co$").unwrap(), ".com"),
    ];
}

pub const SMTP_TIMEOUT: Duration = Duration::from_secs(10);
pub const SMTP_HELO_DOMAIN: &str = "email-verifier.local";
pub const SMTP_SENDER: &str = "verify@email-verifier.local";

pub const MAX_RETRIES: u32 = 3;
pub const RETRY_BACKOFF_BASE: u64 = 2;
pub const RETRY_BACKOFF_MAX: u64 = 30;

pub const MAX_WORKERS: usize = 20;
pub const BATCH_SIZE: usize = 100;
pub const DNS_CACHE_SIZE: usize = 2048;