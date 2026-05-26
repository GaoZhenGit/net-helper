#!/usr/bin/env python3
"""net-helper automated test suite (pipe mode)."""

import subprocess, sys, os

if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')

EXE = os.path.join(os.path.dirname(__file__), "..", "target",
                   "x86_64-pc-windows-gnu", "release", "net-helper.exe")
PASS = FAIL = SKIP = 0

def trim(lines, n=5):
    if len(lines) <= n * 2 + 1:
        return lines
    return lines[:n] + [f"    ... ({len(lines) - n*2} lines omitted) ..."] + lines[-n:]

def test(name, args, stdin=None, must_contain=None, timeout=10):
    global PASS, FAIL, SKIP
    print(f"\n{'─'*50}\n  {name}")

    cmd = [EXE] + args
    inp = stdin.encode() if stdin else None
    try:
        r = subprocess.run(cmd, input=inp, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        print("  FAIL: TIMEOUT"); FAIL += 1; return
    except FileNotFoundError:
        print(f"  SKIP: {EXE} not found"); SKIP += 1; return

    out = r.stdout.decode("utf-8", errors="replace").replace("\r", "")
    err = r.stderr.decode("utf-8", errors="replace").replace("\r", "")

    if stdin:
        lines = [l for l in stdin.strip().split("\n")]
        for l in trim(lines, 3):
            print(f"    > {l}")

    print(f"  [{r.returncode}] ", end="")
    lines = out.strip().split("\n") if out.strip() else []
    if not lines:
        print("(empty)")
    else:
        print("")
        for l in trim(lines):
            if len(l) > 120:
                l = l[:117] + "..."
            print(f"    | {l}")
    if err.strip():
        for l in err.strip().split("\n"):
            print(f"    ERR: {l}")

    ok = True
    if must_contain:
        full = (out + err).lower()
        missed = [m for m in (must_contain if isinstance(must_contain, list) else [must_contain])
                  if m.lower() not in full]
        if missed:
            print(f"  FAIL: not found: {missed}")
            FAIL += 1; return
    print("  PASS"); PASS += 1

# ── info ──────────────────────────────────────────────

test("Version",      ["--version"], must_contain="v2026")
test("Help",         ["-h"],        must_contain="Usage:")
test("Unknown flag", ["--bogus"],   must_contain="Unknown")

# ── DNS ───────────────────────────────────────────────

test("DNS qq.com",    ["-d", "qq.com"],   must_contain=["qq.com", "IPv4"])
test("DNS no args",   ["-d"],             must_contain="Usage:")

# ── UDP ───────────────────────────────────────────────

test("UDP", ["-u", "202.108.144.21", "2077"],
    stdin="usee-test\n/quit",
    must_contain=["[send", "[recv", "usee-test"], timeout=8)

# ── TCP ───────────────────────────────────────────────

http = "GET / HTTP/1.1\nHost: example.com\n\n/quit"
test("TCP HTTP", ["-t", "example.com", "80"],
    stdin=http, must_contain=["[send", "[recv", "200 OK"], timeout=12)

http_eof = "GET / HTTP/1.1\nHost: example.com\n\n"
test("TCP EOF", ["-t", "example.com", "80"],
    stdin=http_eof, must_contain=["[send", "[recv", "200 OK"], timeout=12)

# ── TCP+TLS ───────────────────────────────────────────

tls = "GET / HTTP/1.1\nHost: www.baidu.com\n\n/quit"
test("TCP TLS", ["-t", "-tls", "www.baidu.com", "443"],
    stdin=tls, must_contain=["TLS]", "[send", "[recv", "200 OK"], timeout=15)

# ── summary ───────────────────────────────────────────

total = PASS + FAIL + SKIP
print(f"\n{'='*50}")
print(f"  {PASS} passed  {FAIL} failed  {SKIP} skipped  ({total} total)")
print(f"{'='*50}")
sys.exit(0 if FAIL == 0 else 1)
