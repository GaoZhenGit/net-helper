#!/usr/bin/env python3
"""Local WebSocket echo server for testing net-helper.

Usage:
  python tests/ws_echo.py start    Start WS(9001) + WSS(9002) in background
  python tests/ws_echo.py stop     Stop background servers
  python tests/ws_echo.py          Run WS foreground (default 9878)

Behaviour:
  On connect  → sends "hello ws"
  On message  → replies "[received] <original>"
"""

import asyncio, os, ssl, subprocess, sys, websockets

PID_FILE = os.path.join(os.path.dirname(__file__), ".ws_echo.pid")
CERT_DIR = os.path.dirname(os.path.abspath(__file__))
WS_PORT, WSS_PORT = 9001, 9002

async def handler(ws):
    await ws.send("hello ws")
    async for msg in ws:
        await ws.send(f"[received] {msg}")

def get_ssl_context():
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(os.path.join(CERT_DIR, "cert.pem"), os.path.join(CERT_DIR, "key.pem"))
    return ctx

def kill_pid(pid):
    try:
        if sys.platform == "win32":
            import ctypes
            ctypes.windll.kernel32.TerminateProcess(ctypes.windll.kernel32.OpenProcess(1, 0, pid), 0)
        else:
            os.kill(pid, 15)
    except Exception:
        pass

async def main_forever(port, ssl_flag=False):
    kwargs = {}
    label = "ws"
    if ssl_flag:
        kwargs["ssl"] = get_ssl_context()
        label = "wss"
    async with websockets.serve(handler, "127.0.0.1", port, **kwargs):
        print(f"  {label}://127.0.0.1:{port}/echo")
        await asyncio.Future()

def start_both():
    if os.path.exists(PID_FILE):
        with open(PID_FILE) as f:
            for line in f:
                kill_pid(int(line.strip()))
        os.remove(PID_FILE)
    script = os.path.abspath(__file__)
    pids = []
    for port, ssl_arg in [(WS_PORT, "0"), (WSS_PORT, "1")]:
        p = subprocess.Popen(
            [sys.executable, script, "--daemon", str(port), ssl_arg],
            creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
        )
        pids.append(str(p.pid))
    with open(PID_FILE, "w") as f:
        f.write("\n".join(pids))
    print("WS echo servers started:")
    print(f"  ws://127.0.0.1:{WS_PORT}/echo")
    print(f"  wss://127.0.0.1:{WSS_PORT}/echo (self-signed cert)")

def stop_all():
    if not os.path.exists(PID_FILE):
        print("No server running")
        return
    with open(PID_FILE) as f:
        for line in f:
            kill_pid(int(line.strip()))
    os.remove(PID_FILE)
    print("Servers stopped")

if __name__ == "__main__":
    argv = sys.argv[1:]
    if not argv:
        asyncio.run(main_forever(9878))
    elif argv[0] == "start":
        start_both()
    elif argv[0] == "stop":
        stop_all()
    elif argv[0] == "--daemon":
        port, ssl_int = int(argv[1]), int(argv[2])
        asyncio.run(main_forever(port, ssl_int == 1))
