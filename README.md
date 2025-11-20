# Protobuf Ethics Dataset

## Intent
This repository focuses on collecting, cleaning, and structuring the **ETHICS** dataset for efficient use in generative AI training. It provides tools to export raw data from Hugging Face, clean and prune it, compute statistics, and convert it into dense, machine‑readable formats (JSONL → Protocol Buffers) for model ingestion.

## Purpose
Large language models benefit from curated ethical reasoning data, but the original ETHICS dataset is distributed as CSVs that require cleaning and normalization.  
This project standardizes the full pipeline and provides pruning, statistics, and schema‑based serialization for high‑throughput training.

### ETHICS Dataset Overview
The **ETHICS dataset** (Hendrycks et al., 2021) evaluates a model’s ability to understand human moral reasoning across five subsets: **Commonsense**, **Deontology**, **Virtue Ethics**, **Utilitarianism**, and **Justice**.  
This repository retrieves the dataset from Hugging Face, cleans it, prunes long examples, analyzes distributions, and serializes the data to Protocol Buffers for efficient training.

Dataset source:  
**Hendrycks et al. (2021). ETHICS: Aligning AI With Shared Human Values.**  
https://arxiv.org/abs/2008.02275

---

# Setup & Pipeline

The following steps are ordered to match the actual workflow:
1. **Download raw ETHICS JSONL**
2. **Calculate the text‑length distribution**
3. **Choose a cutoff**
4. **Prune examples exceeding the cutoff**
5. **Convert JSONL → Protobuf**

---

## 1. Create environment

```bash
uv venv .venv
source .venv/bin/activate
uv sync
```

Optional dependency install:

```bash
uv add datasets protobuf grpcio-tools zstandard transformers tokenizers
```

---

## 2. Export ETHICS dataset to JSONL

```bash
uv run python scripts/get_raw_training_data.py --out data/raw
```

Outputs:

```
data/raw/
  commonsense-train.jsonl
  deontology-train.jsonl
  justice-train.jsonl
  utilitarianism-train.jsonl
  virtue-train.jsonl
```

---

## 3. Calculate raw text‑length statistics

This step analyzes character‑length distributions for all commonsense examples and helps determine an appropriate pruning cutoff.

```bash
uv run python scripts/calculate_raw_text_length_stats.py
```

Output summary stored at:

```
data/stats/commonsense_length_stats.toml
```

Use this file to determine the pruning threshold  
(1000 characters is recommended and tested).

---

## 4. Prune the dataset using Rust

Once the cutoff is determined, prune all commonsense JSONL files:

```bash
cargo run --bin prune_data_by_length
```

Or specify only particular files:

```bash
cargo run --bin prune_data_by_length -- data/raw/commonsense-train.jsonl
```

Pruned outputs are written to:

```
data/filtered/
```

Each run prints a summary:

```
commonsense-train.jsonl: kept=8421 dropped=5489 -> data/filtered/commonsense-train.jsonl
```

This step dramatically reduces dataset size and removes lengthy Reddit‑style stories that inflate token count.

---

## 5. Convert JSONL → Protobuf using `ethics-pipeline`

After pruning, the main pipeline converts examples into protobuf shards:

```bash
cargo run --bin ethics-pipeline
```

This tool:

- Reads pruned files from `data/filtered/`
- Fills the schema defined in `proto/ethics.proto`
- Shards output into 32–64 MB chunks
- Compresses using zstd
- Writes to:

```
data/processed/<subset>/<split>-00000.pb.zst
```

These protobuf shards are optimized for training speed and minimal disk I/O.

---

## 6. Generate Python protobuf classes (optional for training)

If training or evaluation happens in Python:

```bash
mkdir -p training/gen
uv run python -m grpc_tools.protoc     -I proto --python_out=training/gen proto/ethics.proto
```

---

# Project Structure

```
/scripts/              # Python exporters, stats analysis, utilities  
/proto/                # Protobuf schema  
/src/                  # Rust pipeline modules  
/src/bin/              # CLI binaries (pruning, main pipeline)  
/training/             # Python training utilities  
/data/
  raw/                 # Unmodified JSONL from ETHICS
  filtered/            # Pruned JSONL (post-cutoff)
  processed/           # Protocol Buffer shards
  stats/               # TOML statistics
```

# Design Principles
- **Schema consistency** – All cleaned data conforms to one protobuf message.
- **Compact encoding** – Protobuf + zstd minimize storage and speed up training.
- **Efficiency** – Long stories removed, shards optimized for streaming.
- **Reproducibility** – Deterministic exports and transparent preprocessing.

---

*Goal: structured ethical‑reasoning data for efficient and reproducible model training.*
