#!/usr/bin/env python3
"""Did a FreeBSD run pass, by SPEC amendments 21 and 25?

Usage:
    python agents/tools/freebsd_ok.py <run-id> [--repo erikerikerik/photocraft]

Prints PASS or FAIL with one line per job, and exits 0 (pass), 1 (fail) or 2 (could not decide: treat as fail).

Amendment 25 (Erik, D18, Oct 19): the **x86_64** job is the required FreeBSD check. A job whose
name contains "aarch64" is **advisory**: its result (success, failure, timeout, still running) is
reported but never blocks. Every other job (x86_64, or the older single job without an
architecture in its name) must be completed and success or skipped, and at least one must have succeeded. The run itself need not be finished while only an aarch64 job is
still running. Standard library and `gh` only; reads GitHub, writes nothing.
"""
import argparse
import json
import subprocess
import sys


def gh(*args):
    out = subprocess.run(["gh", "api", *args], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if out.returncode != 0:
        raise RuntimeError(f"gh api {' '.join(args)}: {out.stderr.strip()}")
    return out.stdout


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run_id")
    ap.add_argument("--repo", default="erikerikerik/photocraft")
    a = ap.parse_args()
    try:
        run = json.loads(gh(f"repos/{a.repo}/actions/runs/{a.run_id}"))
        jobs = json.loads(gh(f"repos/{a.repo}/actions/runs/{a.run_id}/jobs?per_page=100")).get("jobs", [])
        if not jobs:
            print(f"FAIL: run {a.run_id} has no jobs")
            return 2
        verdict, lines, x86_green = 0, [], False
        for j in jobs:
            name, status, concl = j["name"], j.get("status"), j.get("conclusion")
            if "aarch64" in name:
                state = concl if status == "completed" else status
                lines.append(f"  advisory (SPEC 25): {name}: {state}")
                continue
            if status != "completed":
                lines.append(f"  BLOCKS: {name}: still {status}")
                verdict = 1
                continue
            if concl in ("success", "skipped"):
                lines.append(f"  ok: {name}: {concl}")
                if concl == "success":
                    x86_green = True  # any non-aarch64 job: x86_64, or the older single unnamed-arch job
                continue
            lines.append(f"  BLOCKS: {name}: {concl}")
            verdict = 1
        if verdict == 0 and not x86_green:
            lines.append("  BLOCKS: no non-aarch64 FreeBSD job succeeded in this run")
            verdict = 1
        print(("PASS" if verdict == 0 else "FAIL") + f": FreeBSD run {a.run_id} on {run.get('head_sha', '')[:8]}")
        print("\n".join(lines))
        return verdict
    except (RuntimeError, OSError, ValueError, KeyError) as e:
        print(f"FAIL: could not decide ({e}); treat as failed")
        return 2


if __name__ == "__main__":
    sys.exit(main())
