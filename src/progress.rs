use crate::types::{VerificationResult, VerificationStats};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

pub struct ProgressReporter {
    total: usize,
    quiet: bool,
    pbar: Option<ProgressBar>,
    stats: HashMap<String, usize>,
    column_stats: HashMap<String, HashMap<String, usize>>,
    corrections_count: usize,
    start_time: Instant,
}

impl ProgressReporter {
    pub fn new(total: usize, quiet: bool) -> Self {
        let pbar = if !quiet {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };
        
        Self {
            total,
            quiet,
            pbar,
            stats: HashMap::new(),
            column_stats: HashMap::new(),
            corrections_count: 0,
            start_time: Instant::now(),
        }
    }
    
    pub fn update(&mut self, result: &VerificationResult, column_name: &str) {
        let status_str = result.status.as_str().to_string();
        
        *self.stats.entry(status_str.clone()).or_insert(0) += 1;
        self.column_stats
            .entry(column_name.to_string())
            .or_default()
            .entry(status_str)
            .and_modify(|c| *c += 1)
            .or_insert(1);
        
        if result.domain_corrected {
            self.corrections_count += 1;
        }
        
        if let Some(pb) = &self.pbar {
            pb.inc(1);
            
            // Log progress periodically
            if pb.position() % 50 == 0 || pb.position() == self.total as u64 {
                let elapsed = self.start_time.elapsed().as_secs_f64();
                let rate = if elapsed > 0.0 {
                    pb.position() as f64 / elapsed
                } else {
                    0.0
                };
                
                info!(
                    progress = pb.position(),
                    total = self.total,
                    rate = format!("{:.1}", rate),
                    corrections = self.corrections_count,
                    "Progress update"
                );
                
                pb.set_message(format!(
                    "{} corrections | {:.1}/s",
                    self.corrections_count,
                    rate
                ));
            }
        }
    }
    
    pub fn finalize(&mut self) {
        if let Some(pb) = &self.pbar {
            pb.finish_with_message("Done!");
        }
        
        if !self.quiet {
            self.print_summary();
        }
    }
    
    fn print_summary(&self) {
        println!("\n{}", "=".repeat(70));
        println!("EMAIL VERIFICATION SUMMARY");
        println!("{}", "=".repeat(70));
        
        let total: usize = self.stats.values().sum();
        println!("\nTotal email cells processed: {}", total);
        println!("Domain corrections applied: {}", self.corrections_count);
        
        println!("\nStatus Breakdown:");
        let mut sorted: Vec<_> = self.stats.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (status, count) in sorted {
            let pct = if total > 0 {
                (*count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            println!("  {:.<40} {:6} ({:5.1}%)", status, count, pct);
        }
        
        println!("\nPer-Column Breakdown:");
        for (col_name, col_stats) in &self.column_stats {
            println!("\n  {}:", col_name);
            let col_total: usize = col_stats.values().sum();
            let mut sorted: Vec<_> = col_stats.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            for (status, count) in sorted {
                let pct = if col_total > 0 {
                    (*count as f64 / col_total as f64) * 100.0
                } else {
                    0.0
                };
                println!("    {:.<30} {:4} ({:5.1}%)", status, count, pct);
            }
        }
        
        let elapsed = self.start_time.elapsed().as_secs_f64();
        println!("\nTotal time: {:.1} seconds", elapsed);
        if elapsed > 0.0 {
            println!("Average rate: {:.1} emails/second", total as f64 / elapsed);
        }
        println!("{}", "=".repeat(70));
    }
    
    pub fn get_stats(&self) -> VerificationStats {
        VerificationStats {
            total_processed: self.total,
            status_counts: self.stats.clone(),
            column_breakdown: self.column_stats.clone(),
            dedup_stats: Default::default(),
            domain_cache_size: 0,
            corrections_applied: self.corrections_count,
            processing_time_sec: self.start_time.elapsed().as_secs_f64(),
        }
    }
}