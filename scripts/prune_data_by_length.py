"""
Prune JSONL files by filtering out entries with a "text" field character count > CUTOFF.

Usage:
  python scripts/prune_commonsense_jsonl.py           # prunes all files matching COMMONSENSE_GLOB
  python scripts/prune_commonsense_jsonl.py FILES...  # prunes only the given JSONL files

Inputs:
  - One-record-per-line JSONL with a string field "text".

Outputs:
  - Pruned files written to OUTDIR with the same base filename.
  - A summary line per input: "<name>: kept=X dropped=Y -> <outpath>"
"""
import glob
import json
import sys
from pathlib import Path


CUTOFF = 1000  # max length allowed for "text" field (characters, after strip)
COMMONSENSE_GLOB = "data/raw/commonsense-*.jsonl"
OUTDIR = Path("data/filtered")
OUTDIR.mkdir(parents=True, exist_ok=True)


def keep(record: dict) -> bool:
    """Return True if the JSONL record should be kept after pruning.

    A record is kept when:
      - it contains a string field "text", and
      - the character length of text.strip() is <= CUTOFF.

    Args:
        record: A single JSON object parsed from one JSONL line.

    Returns:
        bool: True to keep the record, False to drop it.
    """
    t = record.get("text")
    return isinstance(t, str) and len(t.strip()) <= CUTOFF


# Use provided file args; if none, default to commonsense files
file_args = sys.argv[1:] or glob.glob(COMMONSENSE_GLOB)

for inpath in map(Path, file_args):
    if not inpath.exists():
        print("skip: {} not found".format(inpath), file=sys.stderr)
        continue
    kept = dropped = 0
    outpath = OUTDIR / inpath.name
    with inpath.open("r", encoding="utf-8", errors="ignore") as fin, \
            outpath.open("w", encoding="utf-8") as fout:
        for line in fin:
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
            except json.JSONDecodeError:
                # Malformed JSONL line; skip silently.
                continue
            if keep(data):
                json.dump(data, fout, ensure_ascii=False)
                fout.write("\n")
                kept += 1
            else:
                dropped += 1
    print("{}: kept={} dropped={} -> {}".format(inpath.name, kept, dropped, outpath))
