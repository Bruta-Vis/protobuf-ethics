"""
Scan commonsense JSONL files, compute character-length stats of `text`,
and write results to data/stats/commonsense_length_stats.toml.

Stats per file and overall: count, min, max, mean, std, p25, p50, p75.
"""

import glob
import json
import math
from pathlib import Path

GLOB = "data/raw/commonsense-*.jsonl"
OUT_PATH = Path("data/stats/commonsense_length_stats.toml")
OUT_PATH.parent.mkdir(parents=True, exist_ok=True)


def lengths_from_jsonl(p: Path) -> list[int]:
    """Return a list of character lengths for the `text` field in a JSONL file.

    Each line must be a JSON object with a string `text` field. Malformed lines
    or non-string `text` values are skipped.

    Args:
        p: Path to a JSONL file.

    Returns:
        A list of integer character counts for all valid `text` fields.
    """
    out = []
    with p.open("r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = obj.get("text")
            if isinstance(t, str):
                out.append(len(t))
    return out


def percentile(sorted_vals: list[int], q: float) -> float:
    """Compute the q-th percentile (0–1) via linear interpolation.

    Assumes `sorted_vals` is already sorted ascending. Uses the definition
    that interpolates between the two nearest ranks.

    Args:
        sorted_vals: Sorted list of numeric values.
        q: Percentile in [0.0, 1.0], e.g., 0.25 for p25.

    Returns:
        The percentile value as a float. Returns NaN for empty input.
    """
    if not sorted_vals:
        return math.nan
    n = len(sorted_vals)
    if n == 1:
        return float(sorted_vals[0])
    idx = q * (n - 1)
    lo = int(idx)
    hi = min(lo + 1, n - 1)
    frac = idx - lo
    return sorted_vals[lo] * (1 - frac) + sorted_vals[hi] * frac


def summarize(vals: list[int]) -> dict:
    """Compute descriptive statistics for a list of integer lengths.

    Statistics include: count, min, p25, p50, p75, max, mean, std (sample).

    Args:
        vals: List of integer character counts.

    Returns:
        A dictionary of statistics with float values (NaN for empty input).
    """
    if not vals:
        return {"count": 0, "min": math.nan, "p25": math.nan, "p50": math.nan,
                "p75": math.nan, "max": math.nan, "mean": math.nan, "std": math.nan}
    s = sorted(vals)
    n = len(s)
    mean = sum(s) / n
    var = sum((x - mean) ** 2 for x in s) / (n - 1) if n > 1 else 0.0
    return {
        "count": n,
        "min": float(s[0]),
        "max": float(s[-1]),
        "mean": mean,
        "std": math.sqrt(var),
        "p25": percentile(s, 0.25),
        "p50": percentile(s, 0.50),
        "p75": percentile(s, 0.75),
    }


def fmt_num(v):
    """Format a number for TOML output.

    Converts floats to a compact decimal string, preserving a few decimals and
    rendering NaN as 'nan'. Integers and other types are stringified as-is.

    Args:
        v: Numeric value (int or float).

    Returns:
        A TOML-friendly string representation.
    """
    if isinstance(v, float):
        if math.isnan(v):
            return "nan"
        return f"{v:.2f}".rstrip("0").rstrip(".")
    return str(v)


def write_toml(report: dict, out_path: Path) -> None:
    """Write the aggregated statistics dictionary as TOML.

    Structure:
      [overall]
      count = ...
      ...
      [files."filename.jsonl"]
      count = ...
      ...

    Args:
        report: Dict with keys {"overall": stats, "files": {fname: stats}}.
        out_path: Destination TOML file path.
    """
    lines = ["[overall]"]
    for k in ("count", "min", "max", "mean", "std", "p25", "p50", "p75"):
        lines.append(f'{k} = {fmt_num(report["overall"][k])}')
    lines.append("")

    for fname, stats in sorted(report["files"].items()):
        lines.append(f'[files."{fname}"]')
        for k in ("count", "min", "max", "mean", "std", "p25", "p50", "p75"):
            lines.append(f"{k} = {fmt_num(stats[k])}")
        lines.append("")

    out_path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    """Collect per-file and overall length stats and write a TOML report.

    - Scans all files matching `GLOB`.
    - Computes per-file character-length stats of `text`.
    - Aggregates an overall summary.
    - Writes TOML to `OUT_PATH`.
    """
    files = [Path(p) for p in glob.glob(GLOB)]
    report = {"files": {}, "overall": {}}
    all_lengths: list[int] = []

    for p in files:
        lens = lengths_from_jsonl(p)
        report["files"][p.name] = summarize(lens)
        all_lengths.extend(lens)

    report["overall"] = summarize(all_lengths)
    write_toml(report, OUT_PATH)
    print(f"Wrote {OUT_PATH} with stats for {len(files)} file(s).")


if __name__ == "__main__":
    main()
