/// M8: Artifact caching — hash stage inputs with sha2 + memmap2, skip unchanged stages.
use anyhow::Result;
use colored::Colorize;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::path::Path;

const CACHE_DIR: &str = ".rustpipe-cache";

/// Compute a cache key for a stage: hash its step commands + any source files listed as artifacts.
/// Uses memmap2 for zero-copy file reading.
pub fn stage_hash(stage_name: &str, step_cmds: &[&str], source_files: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(stage_name.as_bytes());
    for cmd in step_cmds {
        hasher.update(cmd.as_bytes());
    }
    for path in source_files {
        if let Ok(mmap) = mmap_file(path) {
            hasher.update(&mmap[..]);
        }
    }
    format!("{:x}", hasher.finalize())
}

fn mmap_file(path: &str) -> Result<Mmap> {
    let file = File::open(path)?;
    // SAFETY: file is read-only and not modified during hashing
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}

/// Returns true if the cache entry for this hash already exists (cache hit).
pub fn is_cached(hash: &str) -> bool {
    Path::new(CACHE_DIR).join(hash).exists()
}

/// Write a cache marker file for this hash.
pub fn write_cache(hash: &str) -> Result<()> {
    fs::create_dir_all(CACHE_DIR)?;
    fs::write(Path::new(CACHE_DIR).join(hash), b"")?;
    Ok(())
}

/// Remove all cache entries.
pub fn clear() -> Result<()> {
    if Path::new(CACHE_DIR).exists() {
        fs::remove_dir_all(CACHE_DIR)?;
    }
    println!("{} Cache cleared ({})", "✓".green().bold(), CACHE_DIR.cyan());
    Ok(())
}

/// Check cache and print hit/miss. Returns true if stage should be skipped.
pub fn check(stage_name: &str, hash: &str) -> bool {
    if is_cached(hash) {
        println!(
            "  {} Stage {} {} (hash: {})",
            "⚡".yellow(),
            stage_name.cyan().bold(),
            "CACHE HIT — skipping".green().bold(),
            &hash[..12]
        );
        true
    } else {
        println!(
            "  {} Stage {} {} (hash: {})",
            "○".dimmed(),
            stage_name.cyan(),
            "cache miss".dimmed(),
            &hash[..12]
        );
        false
    }
}
