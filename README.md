# Protobuf Ethics Dataset

## Intent
This repository focuses on collecting, cleaning, and structuring the **ETHICS** dataset for efficient use in generative AI training. It provides tools to export raw data from Hugging Face, clean and prune it, compute statistics, and convert it into dense, machine‑readable formats (JSONL → Protocol Buffers) for model ingestion.

## Purpose
Large language models benefit from curated ethical reasoning data, but the original ETHICS dataset is distributed as CSVs that require cleaning and normalization.
This project standardizes the full pipeline and provides pruning, statistics, and schema‑based serialization for high‑throughput training.

### ETHICS Dataset Overview
The **ETHICS dataset** (Hendrycks et al., 2021) evaluates a model’s ability to understand human moral reasoning across five subsets: **Commonsense**, **Deontology**, **Virtue Ethics**, **Utilitarianism**, and **Justice**.
This repository retrieves the dataset from Hugging Face, cleans it, prunes long examples, analyzes distributions, and serializes the data to Protocol Buffers for efficient training.

Dataset source:  
**Hendrycks et al. (2021). ETHICS: Aligning AI With Shared Human Values.**  
https://arxiv.org/abs/2008.02275

---

# Setup & Pipeline

Steps are ordered to match the actual workflow:
1. **Download raw ETHICS csv and format it as JSONL**
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
  commonsense-test.jsonl
  commonsense-test_hard.jsonl
  deontology-train.jsonl
  deontology-test.jsonl
  deontology-test_hard.jsonl
  justice-train.jsonl
  justice-test.jsonl
  justice-test_hard.jsonl
  utilitarianism-train.jsonl
  utilitarianism-test.jsonl
  utilitarianism-test_hard.jsonl
  virtue-train.jsonl
  virtue-test.jsonl
  virtue-test_hard.jsonl
```

---

## 3. Calculate raw text‑length statistics (Rust)

Analyzes character‑length distributions for all commonsense examples to determine a pruning cutoff.

```bash
cargo run --bin calculate_text_length_stats
```

Output written to:

```
data/stats/commonsense_length_stats.toml
```

Use this file to choose a cutoff  
(1,000 characters recommended).

---

## 4. Prune dataset with Rust

Prune all commonsense JSONL files:

```bash
cargo run --bin prune_data_by_length
```

Or prune specific files:

```bash
cargo run --bin prune_data_by_length -- data/raw/commonsense-train.jsonl
```

Pruned output is saved in:

```
data/filtered/
```

---

## 5. Convert JSONL → Protobuf (`ethics-pipeline`)

```bash
cargo run --bin ethics-pipeline
```

This pipeline:

- Reads from `data/filtered/`
- Applies the schema in `proto/ethics.proto`
- Shards into 32–64 MB protobuf files
- Compresses with zstd
- Writes to:

```
data/processed/<subset>/<split>-00000.pb.zst
```

Optimized for training throughput.

---

## 6. (Optional) Generate Python protobuf classes

```bash
mkdir -p training/gen
uv run python -m grpc_tools.protoc -I proto --python_out=training/gen proto/ethics.proto
```

---

# Project Structure

```
/scripts/              # Python exporters & utilities  
/proto/                # Protobuf schema  
/src/                  # Rust modules  
/src/bin/              # CLI tools  
/training/             # Python helpers  
/data/
  raw/                 # Raw JSONL  
  filtered/            # Pruned JSONL  
  processed/           # Protobuf shards  
  stats/               # Length statistics  
```

# Design Principles
- **Schema consistency**
- **Compact encoding (protobuf + zstd)**
- **Efficiency (pruning removes long low‑value stories)**
- **Reproducibility**

---

*Goal: structured ethical‑reasoning data for efficient and reproducible model training.*
