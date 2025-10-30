use anyhow::*;
use prost::Message;
use serde::Deserialize;
use std::{fs::File, io::{BufRead, BufReader, Write}};
use zstd::stream::write::Encoder as ZstdEncoder;

pub mod ethics { include!(concat!(env!("OUT_DIR"), "/ethics.v1.rs")); }
use ethics::Example;

#[derive(Deserialize)]
struct Row {
    #[serde(default)] scenario: String,
    #[serde(default)] question: String,
    #[serde(default)] observation: String,
    #[serde(default)] label: i32,
    #[serde(flatten)] rest: serde_json::Value, // capture anything else
}

fn pick_text(r: &Row) -> String {
    if !r.scenario.is_empty() { r.scenario.clone() }
    else if !r.question.is_empty() { r.question.clone() }
    else { r.observation.clone() }
}

fn jsonl_to_pb(input: &str, subset: &str, split: &str, out_pbzst: &str) -> Result<()> {
    let f = File::open(input)?;
    let mut enc = ZstdEncoder::new(File::create(out_pbzst)?, 9)?; // zstd level 9
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let row: Row = serde_json::from_str(&line)?;

        let mut ex = Example {
            subset: subset.to_string(),
            split:  split.to_string(),
            text:   pick_text(&row),
            label:  row.label,
            meta:   Default::default(),
        };

        if let Some(obj) = row.rest.as_object() {
            for (k, v) in obj {
                if ["rationale","action","answer","input","output"].contains(&k.as_str()) {
                    ex.meta.insert(k.clone(), v.to_string());
                }
            }
        }

        let mut buf = Vec::with_capacity(ex.encoded_len());
        ex.encode_length_delimited(&mut buf)?;
        enc.write_all(&buf)?;
    }
    enc.finish()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    jsonl_to_pb("data/virtue-train.jsonl", "virtue", "train", "shards/virtue-train.pb.zst")?;
    Ok(())
}
