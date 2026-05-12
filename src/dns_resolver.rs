use crate::config::DNS_CACHE_SIZE;
use crate::types::DomainInfo;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex; // only used for the LRU cache, never held across .await
use lazy_static::lazy_static;
use anyhow::Result;

lazy_static! {
    // The LRU cache uses a plain std::sync::Mutex but we NEVER hold the lock
    // across an .await point — we clone what we need, then drop the lock.
    static ref DNS_CACHE: Mutex<LruCache<String, DomainInfo>> =
        Mutex::new(LruCache::new(NonZeroUsize::new(DNS_CACHE_SIZE).unwrap()));

    static ref DNS_RESOLVER: TokioAsyncResolver =
        TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
}

pub async fn resolve_mx_records_cached(domain: &str) -> Result<(Vec<String>, bool)> {
    // ── 1. Check LRU cache (lock acquired + released BEFORE any .await) ───────
    let cached_mx: Option<Vec<String>> = {
        let cache = DNS_CACHE.lock().unwrap();
        cache
            .peek(domain)
            .filter(|info| !info.mx_records.is_empty())
            .map(|info| info.mx_records.clone())
    }; // lock released here

    if let Some(mx_records) = cached_mx {
        return Ok((mx_records, true));
    }

    // ── 2. Async DNS lookup (no lock held) ────────────────────────────────────
    let mx_records: Vec<String> = match DNS_RESOLVER.mx_lookup(domain).await {
        Ok(lookup) => {
            let mut records: Vec<(u16, String)> = lookup
                .iter()
                .map(|mx| (mx.preference(), mx.exchange().to_string()))
                .collect();
            records.sort_by_key(|r| r.0);
            records
                .into_iter()
                .map(|(_, host)| host.trim_end_matches('.').to_string())
                .collect()
        }
        Err(_) => {
            // Fallback to A record
            match DNS_RESOLVER.lookup_ip(domain).await {
                Ok(_) => vec![domain.to_string()],
                Err(_) => vec![],
            }
        }
    };

    let success = !mx_records.is_empty();

    // ── 3. Update cache (lock acquired + released AFTER .await) ───────────────
    {
        let mut cache = DNS_CACHE.lock().unwrap();
        let mut info = cache
            .get(domain)
            .cloned()
            .unwrap_or_else(|| DomainInfo::new(domain.to_string()));
        info.mx_records = mx_records.clone();
        info.check_count += 1;
        info.last_checked = chrono::Utc::now();
        cache.put(domain.to_string(), info);
    } // lock released here

    Ok((mx_records, success))
}

pub fn check_disposable_domain(domain: &str) -> bool {
    crate::config::DISPOSABLE_DOMAINS.contains(domain.to_lowercase().as_str())
}

pub fn check_role_based_address(email: &str) -> bool {
    if let Some(local_part) = email.split('@').next() {
        let local_lower = local_part.to_lowercase();
        crate::config::ROLE_BASED_PREFIXES.iter().any(|prefix| {
            local_lower == *prefix || local_lower.starts_with(&format!("{}.", prefix))
        })
    } else {
        false
    }
}