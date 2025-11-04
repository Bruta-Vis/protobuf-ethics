import argparse
import csv
import io
import json
import os
import sys
import urllib.request

BASE = "https://huggingface.co/datasets/hendrycks/ethics/resolve/main/data"


def select_commonsense(row):
    text = row.get("input") or row.get("text") or row.get("scenario") or ""
    return {"text": text, "label": int(row["label"])}


def select_deontology(row):
    return {"scenario": row.get("scenario", "") or row.get("text", ""),
            "label": int(row["label"]),
            "excuse": row.get("excuse", "")}


def select_justice(row):
    return {"scenario": row.get("scenario", "") or row.get("text", ""),
            "label": int(row["label"])}


def select_utilitarianism(row):
    # headers: baseline,less_pleasant
    return {"baseline": row.get("baseline", ""), "less_pleasant": row.get("less_pleasant", "")}


def select_virtue(row):
    return {"scenario": row.get("scenario", "") or row.get("text", ""),
            "label": int(row["label"])}


SELECTORS = {
    "commonsense": select_commonsense,
    "deontology": select_deontology,
    "justice": select_justice,
    "utilitarianism": select_utilitarianism,
    "virtue": select_virtue,
}


def stream_csv(url):
    # decode once, preserve quoted commas and newlines
    with urllib.request.urlopen(url) as r:
        text = r.read().decode("utf-8", errors="replace")
    # DictReader handles headers and quoting
    yield from csv.DictReader(io.StringIO(text))


def export_subset(outdir, subset, split):
    os.makedirs(outdir, exist_ok=True)
    url = f"{BASE}/{subset}/{split}.csv"
    selector = SELECTORS[subset]
    out_path = os.path.join(outdir, f"{subset}-{split}.jsonl")
    ok, skipped = 0, 0
    with open(out_path, "w", encoding="utf-8") as out:
        try:
            for row in stream_csv(url):
                try:
                    if not row or all(v == "" for v in row.values()):
                        skipped += 1
                        continue
                    rec = {"subset": subset, "split": split, **selector(row)}
                    # minimal validation
                    if "label" in rec and rec["label"] not in (0, 1, 2, 3, 4, 5):
                        # some subsets only use 0/1; adjust if needed
                        pass
                    json.dump(rec, out, ensure_ascii=False)
                    out.write("\n")
                    ok += 1
                except Exception:
                    skipped += 1
        except Exception as e:
            print(f"skip {subset}/{split}: {e}", file=sys.stderr)
    print(f"wrote {out_path}  ok={ok} skipped={skipped}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    for subset in SELECTORS:
        for split in ("train", "test", "test_hard"):
            export_subset(args.out, subset, split)


if __name__ == "__main__":
    main()
