use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use glob::glob;
use serde::Serialize;
use serde_json::Value;
use tracing::{info, warn};

/// Newtype for text length in bytes.
#[derive(Debug, Clone, Copy)]
struct TextLen(usize);

/// Per-file / overall statistics.
#[derive(Debug, Clone, Serialize)]
struct Stats {
    count: usize,
    min: Option<f64>,
    max: Option<f64>,
    mean: Option<f64>,
    std: Option<f64>,
    p25: Option<f64>,
    p50: Option<f64>,
    p75: Option<f64>,
}

/// Top-level TOML structure.
#[derive(Debug, Serialize)]
struct Report {
    overall: Stats,
    files: BTreeMap<String, Stats>,
}

/// Streaming aggregator for overall stats (mean/std/min/max).
#[derive(Debug, Default)]
struct RunningStats {
    count: usize,
    mean: f64,
    m2: f64, // sum of squared deviations
    min: Option<usize>,
    max: Option<usize>,
}

impl RunningStats {
    fn push(&mut self, len: TextLen) {
        let x = len.0;
        // update count, min, max
        self.count += 1;
        self.min = Some(self.min.map_or(x, |m| m.min(x)));
        self.max = Some(self.max.map_or(x, |m| m.max(x)));

        // Welford's online algorithm for mean/std
        let xf = x as f64;
        let delta = xf - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = xf - self.mean;
        self.m2 += delta * delta2;
    }

    fn finalize(self, p25: Option<f64>, p50: Option<f64>, p75: Option<f64>) -> Stats {
        if self.count == 0 {
            return Stats {
                count: 0,
                min: None,
                max: None,
                mean: None,
                std: None,
                p25,
                p50,
                p75,
            };
        }

        let var = if self.count > 1 {
            self.m2 / (self.count as f64 - 1.0)
        } else {
            0.0
        };

        Stats {
            count: self.count,
            min: self.min.map(|v| v as f64),
            max: self.max.map(|v| v as f64),
            mean: Some(self.mean),
            std: Some(var.sqrt()),
            p25,
            p50,
            p75,
        }
    }
}

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(
    name = "calculate-text-length-stats",
    about = "Compute per-file and overall text-length statistics from JSONL files."
)]
struct Args {
    #[arg(
        long,
        default_value = "data/raw/commonsense-*.jsonl",
        value_name = "GLOB"
    )]
    glob: String,

    #[arg(
        long,
        default_value = "data/stats/commonsense_length_stats.toml",
        value_name = "OUT"
    )]
    out: String,
}

fn lengths_from_jsonl(path: &Path) -> Result<Vec<TextLen>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open JSONL file {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut out = Vec::new();

    for line_result in reader.lines() {
        let line = line_result
            .with_context(|| format!("error reading line from {}", path.display()))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let obj: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                continue;
            }
        };

        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
            // Use byte length for efficiency; suitable proxy for token count here.
            let len = text.len();
            out.push(TextLen(len));
        }
    }

    Ok(out)
}

fn percentile(sorted_vals: &[TextLen], q: f64) -> Option<f64> {
    if sorted_vals.is_empty() {
        return None;
    }
    let n = sorted_vals.len();
    if n == 1 {
        return Some(sorted_vals[0].0 as f64);
    }

    let idx = q * (n as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx - lo as f64;

    let lo_val = sorted_vals[lo].0 as f64;
    let hi_val = sorted_vals[hi].0 as f64;

    Some(lo_val * (1.0 - frac) + hi_val * frac)
}

fn summarize_per_file(vals: &[TextLen]) -> Stats {
    if vals.is_empty() {
        return Stats {
            count: 0,
            min: None,
            max: None,
            mean: None,
            std: None,
            p25: None,
            p50: None,
            p75: None,
        };
    }

    let mut s = vals.to_vec();
    s.sort_unstable_by_key(|x| x.0);

    let n = s.len();
    let sum: f64 = s.iter().map(|&x| x.0 as f64).sum();
    let mean = sum / n as f64;

    let var = if n > 1 {
        let mut acc = 0.0;
        for &x in &s {
            let dx = x.0 as f64 - mean;
            acc += dx * dx;
        }
        acc / (n as f64 - 1.0)
    } else {
        0.0
    };

    Stats {
        count: n,
        min: Some(s[0].0 as f64),
        max: Some(s[n - 1].0 as f64),
        mean: Some(mean),
        std: Some(var.sqrt()),
        p25: percentile(&s, 0.25),
        p50: percentile(&s, 0.50),
        p75: percentile(&s, 0.75),
    }
}

fn run(args: Args) -> Result<()> {
    // Find input files by glob.
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in glob(&args.glob).with_context(|| format!("invalid glob: {}", args.glob))? {
        match entry {
            Ok(path) => files.push(path),
            Err(e) => warn!("glob match error: {e}"),
        }
    }

    if files.is_empty() {
        warn!("No files matched pattern: {}", args.glob);
    } else {
        info!("Found {} file(s) for pattern {}", files.len(), args.glob);
    }

    let mut file_stats: BTreeMap<String, Stats> = BTreeMap::new();
    let mut overall_lengths: Vec<TextLen> = Vec::new();
    let mut overall_running = RunningStats::default();

    for path in &files {
        info!("Processing {}", path.display());
        let lens = lengths_from_jsonl(path)?;
        let stats = summarize_per_file(&lens);

        // Add per-file stats.
        let fname = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        file_stats.insert(fname, stats);

        // Feed lengths into overall running stats + global vector for percentiles.
        for len in &lens {
            overall_running.push(*len);
        }
        overall_lengths.extend(lens);
    }

    // Compute overall percentiles once, from sorted global lengths.
    overall_lengths.sort_unstable_by_key(|x| x.0);
    let p25 = percentile(&overall_lengths, 0.25);
    let p50 = percentile(&overall_lengths, 0.50);
    let p75 = percentile(&overall_lengths, 0.75);

    let overall = overall_running.finalize(p25, p50, p75);

    // Build and write report.
    let report = Report {
        overall,
        files: file_stats,
    };

    let out_path = PathBuf::from(&args.out);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent dir {}", parent.display()))?;
    }

    let toml_str = toml::to_string_pretty(&report)
        .context("failed to serialize statistics report to TOML")?;
    std::fs::write(&out_path, toml_str)
        .with_context(|| format!("failed to write TOML report to {}", out_path.display()))?;

    info!(
        "Wrote {} with stats for {} file(s).",
        out_path.display(),
        report.files.len()
    );

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    run(args)
}
