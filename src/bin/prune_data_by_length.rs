use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use glob::glob;
use serde_json::Value;

const CUTOFF: usize = 1000;
const COMMONSENSE_GLOB: &str = "data/raw/commonsense-*.jsonl";
const OUTDIR: &str = "data/filtered";

fn keep(record: &Value) -> bool {
    let Some(text) = record.get("text").and_then(|v| v.as_str()) else {
        return false;
    };

    text.trim().chars().count() <= CUTOFF
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(OUTDIR)?;

    let args: Vec<String> = env::args().skip(1).collect();

    let input_paths: Vec<PathBuf> = if args.is_empty() {
        let mut paths = Vec::new();
        for entry in glob(COMMONSENSE_GLOB)? {
            if let Ok(path) = entry {
                paths.push(path);
            }
        }
        paths
    } else {
        args.into_iter().map(PathBuf::from).collect()
    };

    for inpath in input_paths {
        if !inpath.exists() {
            eprintln!("skip: {} not found", inpath.display());
            continue;
        }

        let file_name = match inpath.file_name() {
            Some(name) => name.to_os_string(),
            None => {
                eprintln!("skip: {} has no file name", inpath.display());
                continue;
            }
        };

        let outpath = Path::new(OUTDIR).join(file_name);

        let fin = File::open(&inpath)?;
        let reader = BufReader::new(fin);

        let fout = File::create(&outpath)?;
        let mut writer = BufWriter::new(fout);

        let mut kept: usize = 0;
        let mut dropped: usize = 0;

        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let record: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    continue;
                }
            };

            if keep(&record) {
                writer.write_all(trimmed.as_bytes())?;
                writer.write_all(b"\n")?;
                kept += 1;
            } else {
                dropped += 1;
            }
        }

        writer.flush()?;

        println!(
            "{}: kept={} dropped={} -> {}",
            inpath
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            kept,
            dropped,
            outpath.display()
        );
    }

    Ok(())
}
