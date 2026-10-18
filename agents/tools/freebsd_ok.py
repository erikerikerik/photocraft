#!/usr/bin/env python3
"""Did a FreeBSD run pass, by SPEC amendments 21 and 24?

Usage:
    python agents/tools/freebsd_ok.py <run-id> [--repo erikerikerik/photocraft]

Prints PASS or FAIL with one reason per job, and exits 0 (pass), 1 (fail) or 2 (could not decide: treat as fail).

A run passes when it is completed and every job is success or skipped, with one exception
(amendment 24, tracked as FB-01): a job whose name contains "aarch64" may fail **only** when its
log shows Rust's installer refusing the platform ("installer for platform 'aarch64-unknown-freebsd'
not found"), and only when an x86_64 job in the same run succeeded. Any other failure fails the run.
Standard library and `gh` only; reads GitHub, writes nothing.
"""
import argparse
import json
import subprocess
import sys

INSTALLER_MISSING = "installer for platform 'aarch64-unknown-freebsd' not found"


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
        if run.get("status") != "completed":
            print(f"FAIL: run {a.run_id} is {run.get('status')}, not completed")
            return 1
        jobs = json.loads(gh(f"repos/{a.repo}/actions/runs/{a.run_id}/jobs?per_page=100")).get("jobs", [])
        if not jobs:
            print(f"FAIL: run {a.run_id} has no jobs")
            return 2
        x86_green = any("x86_64" in j["name"] and j.get("conclusion") == "success" for j in jobs)
        verdict, lines = 0, []
        for j in jobs:
            name, concl = j["name"], j.get("conclusion")
            if concl in ("success", "skipped"):
                lines.append(f"  ok: {name}: {concl}")
                continue
            if "aarch64" in name and concl == "failure":
                log = gh(f"repos/{a.repo}/actions/jobs/{j['id']}/logs")
                if INSTALLER_MISSING in log and x86_green:
                    lines.append(f"  excused (SPEC 24, FB-01): {name}: Rust installer has no aarch64-unknown-freebsd build")
                    continue
                why = "x86_64 is not green in this run" if INSTALLER_MISSING in log else "it failed somewhere other than the Rust installer"
                lines.append(f"  BLOCKS: {name}: failure, not excused because {why}")
                verdict = 1
                continue
            lines.append(f"  BLOCKS: {name}: {concl}")
            verdict = 1
        print(("PASS" if verdict == 0 else "FAIL") + f": FreeBSD run {a.run_id} on {run.get('head_sha', '')[:8]}")
        print("\n".join(lines))
        return verdict
    except (RuntimeError, OSError, ValueError, KeyError) as e:
        print(f"FAIL: could not decide ({e}); treat as failed")
        return 2


if __name__ == "__main__":
    sys.exit(main())
