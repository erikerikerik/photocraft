#!/usr/bin/env python3
"""Does this change need a FreeBSD run before it merges? (SPEC amendment 21)

Usage (from a checkout of the branch):
    python agents/tools/needs_freebsd.py [--base origin/main] [--head HEAD] [--branch NAME]

Exit 0 = not needed (platform-independent change), 10 = needed (reasons are printed),
2 = could not decide (treat as needed). Standard library only; reads git, writes nothing.

FreeBSD is needed for:
- upstream syncs (branch sync/*) and upstream-bound fixes (branch upfix/*, task UP-xx);
- dependency, build, CI or packaging changes;
- code that is platform-specific: cfg(...) on an OS or family, std::os, sockets, processes,
  file permissions, environment variables, paths to OS folders.
When in doubt it says "needed": a missed FreeBSD regression costs more than one slow CI run.
"""
import argparse
import re
import subprocess
import sys

# Any change to these paths needs FreeBSD.
SENSITIVE_PATHS = [
    (re.compile(r"(^|/)Cargo\.(toml|lock)$"), "dependencies or workspace (Cargo.toml / Cargo.lock)"),
    (re.compile(r"^\.github/"), "CI workflows"),
    (re.compile(r"(^|/)build\.rs$"), "a build script"),
    (re.compile(r"^packaging/"), "packaging"),
    (re.compile(r"^xtask/"), "xtask (build tooling)"),
    (re.compile(r"^\.cargo/|(^|/)rust-toolchain(\.toml)?$|(^|/)deny\.toml$"), "toolchain or cargo config"),
    (re.compile(r"^crates/ai-client/"), "the AI client (sockets, endpoint paths, file permissions)"),
]

# Paths that never need it on their own (docs and agent notes).
DOC_PATHS = re.compile(r"^(agents/|docs/|book/)|\.md$|\.txt$|\.tsv$|\.png$|\.svg$")

# Changed source lines matching these need FreeBSD.
SENSITIVE_LINES = [
    (re.compile(r"cfg!?\s*\(.*\b(unix|windows|target_os|target_family|target_vendor|target_env)\b"), "platform cfg"),
    (re.compile(r"\bstd::os::|\bstd::os\b|\blibc::|\bwindows_sys::|\bwinapi::|\bnix::"), "OS-specific API"),
    (re.compile(r"\bstd::net\b|\bTcpStream\b|\bTcpListener\b|\bUdpSocket\b|\bset_nonblocking\b"), "sockets"),
    (re.compile(r"\bstd::process\b|\bCommand::new\b"), "processes"),
    (re.compile(r"\bset_permissions\b|\bPermissionsExt\b|\bOpenOptionsExt\b|\.mode\(\s*0o"), "file permissions"),
    (re.compile(r"\benv::var\b|\bvar_os\b|\bLOCALAPPDATA\b|\bXDG_[A-Z_]+\b|\bHOME\b|\bdirs::"), "environment or OS folders"),
]


def git(*args):
    out = subprocess.run(["git", *args], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {out.stderr.strip()}")
    return out.stdout


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="origin/main")
    ap.add_argument("--head", default="HEAD")
    ap.add_argument("--branch", default=None, help="branch name; defaults to the current branch")
    a = ap.parse_args()
    reasons = []
    try:
        branch = a.branch or git("rev-parse", "--abbrev-ref", a.head).strip()
        if branch.startswith(("sync/", "upfix/")) or re.search(r"(^|/)(up|sy)-\d+", branch, re.I):
            reasons.append(f"branch {branch}: upstream sync or upstream-bound fix")
        files = [f for f in git("diff", "--name-only", f"{a.base}...{a.head}").splitlines() if f]
        for f in files:
            for rx, why in SENSITIVE_PATHS:
                if rx.search(f):
                    reasons.append(f"{f}: {why}")
                    break
        code = [f for f in files if not DOC_PATHS.search(f)]
        if code:
            diff = git("diff", "-U0", f"{a.base}...{a.head}", "--", *code)
            current = None
            for line in diff.splitlines():
                if line.startswith("+++ "):
                    current = line[6:] if line.startswith("+++ b/") else line[4:]
                    continue
                if not line.startswith(("+", "-")) or line.startswith(("+++", "---")):
                    continue
                for rx, why in SENSITIVE_LINES:
                    if rx.search(line[1:]):
                        reasons.append(f"{current}: {why}: {line[1:].strip()[:100]}")
                        break
    except (RuntimeError, OSError) as e:
        print(f"needs-freebsd: could not decide ({e}); treat as NEEDED")
        return 2
    if reasons:
        print("needs-freebsd: NEEDED")
        seen = set()
        for r in reasons:
            if r not in seen:
                seen.add(r)
                print(f"  - {r}")
        return 10
    print("needs-freebsd: not needed (platform-independent change)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
