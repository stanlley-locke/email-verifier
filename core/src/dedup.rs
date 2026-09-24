use crate::types::{VerificationResult, DedupStats};
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct DeduplicationManager {
    verified: Arc<RwLock<HashMap<String, VerificationResult>>>,
    pending: Arc<RwLock<HashSet<String>>>,
}

impl DeduplicationManager {
    pub fn new() -> Self {
        Self {
            verified: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    pub async fn get_or_queue(&self, email: &str) -> Option<VerificationResult> {
        // Check if already verified
        {
            let verified = self.verified.read().await;
            if let Some(result) = verified.get(email) {
                return Some(result.clone());
            }
        }
        
        // Check if pending
        {
            let mut pending = self.pending.write().await;
            if pending.contains(email) {
                return None; // Already being processed
            }
            pending.insert(email.to_string());
        }
        
        None
    }
    
    pub async fn store_result(&self, email: String, result: VerificationResult) {
        {
            let mut verified = self.verified.write().await;
            verified.insert(email.clone(), result);
        }
        {
            let mut pending = self.pending.write().await;
            pending.remove(&email);
        }
    }
    
    pub async fn get_stats(&self) -> DedupStats {
        let verified = self.verified.read().await;
        let pending = self.pending.read().await;
        
        DedupStats {
            total_verified: verified.len(),
            pending: pending.len(),
            dedup_savings: verified.len(),
        }
    }
}

impl Default for DeduplicationManager {
    fn default() -> Self {
        Self::new()
    }
}