# Protobuf Ethics Dataset

## Intent
This repository focuses on collecting, cleaning, and structuring the **ETHICS** dataset for efficient use in generative AI training.  
It provides tools to export raw data from Hugging Face, convert it into dense, machine-readable formats (JSONL → Protocol Buffers), and prepare it for model ingestion.

## Purpose
Large language models benefit from curated ethical reasoning data, but the original ETHICS dataset is distributed as CSVs that require cleaning and normalization.  
This project standardizes the pipeline to transform ETHICS into a compact, schema-defined format suitable for high-throughput training pipelines.


### ETHICS Dataset Overview
The **ETHICS dataset** (Hendrycks et al., 2021) is a large-scale benchmark designed to evaluate a model’s understanding of human moral reasoning. It contains five complementary subsets—**Commonsense**, **Deontology**, **Virtue Ethics**, **Utilitarianism**, and **Justice**—each consisting of short text scenarios paired with human moral judgments. These tasks test a model’s ability to reason about duties, virtues, outcomes, and social norms rather than factual recall. ETHICS has become a standard reference for research in AI alignment and moral reasoning. In this repository, the dataset is pulled from [Hugging Face](https://huggingface.co/datasets/hendrycks/ethics) and transformed into structured JSONL and Protocol Buffer formats for efficient training and reproducibility.

---

## Setup

### 1. Create environment
```bash
uv venv .venv
source .venv/bin/activate
uv sync
```

### 2. Install dependencies manually (if needed)
```bash
uv add datasets protobuf grpcio-tools zstandard transformers tokenizers
```

### 3. Generate Python protobuf classes
```bash
mkdir -p training/gen
uv run python -m grpc_tools.protoc -I proto --python_out=training/gen proto/ethics.proto
```

### 4. Export ETHICS dataset to JSONL
Run the Python exporter to fetch and clean the ETHICS CSVs from Hugging Face.

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

### 5. Convert JSONL to Protobuf (Rust pipeline)
After exporting, the Rust pipeline reads from `data/raw/` and writes compressed protobuf shards to `data/processed/`.

```bash
cargo run
```

---

### 6. Calculate raw text length statistics

Use this script to analyze the character-length distribution of the `text` field across all `commonsense` JSONL files in `data/raw/`.
It outputs per-file and overall statistics (count, min, max, mean, std, p25, p50, p75) to a TOML file:

```
data/stats/commonsense_length_stats.toml
```

#### Run manually
```bash
python scripts/calculate_raw_text_length_stats.py
```

#### Example output
```toml
[overall]
count = 21759
min = 10
max = 12198
mean = 971.80
std = 1082.11
p25 = 56
p50 = 609
p75 = 1719

[files."commonsense-train.jsonl"]
count = 13910
min = 10
max = 12198
mean = 1023.48
std = 1141.82
p25 = 56
p50 = 667
p75 = 1811
```

#### Purpose
These statistics help determine optimal pruning thresholds for text length (in characters) to balance dataset quality and model efficiency.
For example, the default 1,000-character cutoff retains roughly 60% of examples while removing long Reddit-style stories that inflate token count.

---

### Structure
```
/scripts/              # Python exporters
/proto/                # Protobuf schemas
/src/                  # Rust pipeline
/training/             # Training utilities and generated protobuf classes
/data/
  raw/                 # JSONL dumps from ETHICS subsets
  processed/           # Serialized protobuf shards
```

## Design Principles
- **Schema consistency** – all subsets conform to a unified `Example` message.
- **Compact encoding** – Protocol Buffers with zstd compression minimize I/O and storage.
- **Reproducibility** – deterministic exports and stable field numbering.
- **Language-agnostic format** – Protocol Buffers schema usable from any runtime; reference pipeline in Rust and training in Python.

## Usage
Use this repository to prepare and serialize the ETHICS dataset for downstream model training or evaluation tasks.  
It provides a clean, versioned data flow from raw Hugging Face CSVs to ready-to-train protobuf shards.

---
*Goal: Structured ethical reasoning data for efficient and reproducible model training.*




Dataset source: [ETHICS: Aligning AI With Shared Human Values](https://arxiv.org/abs/2008.02275) —  
Dan Hendrycks, Collin Burns, Steven Basart, Andrew Critch, Jerry Li, Dawn Song, and Jacob Steinhardt (2021, ICLR).
