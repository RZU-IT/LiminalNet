import json
import os
import pathlib
import socket
import subprocess
import sys
import tempfile
import time

binary = pathlib.Path(sys.argv[1]).resolve()
version = sys.argv[2]
adapter = "\\Device\\NPF_Loopback" if os.name == "nt" else "lo"
process_flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)
with tempfile.TemporaryDirectory(prefix="liminalnet-install-test-") as folder:
    folder = pathlib.Path(folder)
    env = dict(os.environ, LIMINALNET_CONFIG_DIR=str(folder / "profile"))
    def run(*args):
        result = subprocess.run([str(binary), *map(str, args)], env=env, capture_output=True,
                                text=True, encoding="utf-8", timeout=15,
                                creationflags=process_flags)
        assert result.returncode == 0, result.stderr
        return result.stdout

    assert run("--version").strip() == f"liminalnet {version}"
    assert adapter in run("--list-interfaces")
    about = run("--render", "about")
    assert "Experimental build" in about
    assert "github.com/RZU-IT/LiminalNet" in about
    assert "Buy Me a Coffee" not in about
    for screen in ("sources", "settings", "overview", "inspect"):
        assert "LiminalNet" in run("--render", screen)

    recording = folder / "loopback.pcap"
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as receiver, socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sender:
        receiver.bind(("127.0.0.1", 0))
        receiver.settimeout(1)
        port = receiver.getsockname()[1]
        process = subprocess.Popen([str(binary), "--interface", adapter,
                                    "--filter", f"udp dst port {port}", "--record", str(recording),
                                    "--capture-seconds", "5"], env=env, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, text=True, encoding="utf-8",
                                    creationflags=process_flags)
        try:
            time.sleep(1)
            for _ in range(40):
                sender.sendto(b"LiminalNet installation test", ("127.0.0.1", port))
                receiver.recvfrom(1024)
                time.sleep(0.025)
            stdout, stderr = process.communicate(timeout=12)
            assert process.returncode == 0, stderr
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    live = json.loads(stdout)
    packets = sum(flow["packets"] for flow in live["flows"])
    assert packets >= 40, live
    replay = json.loads(run("--read", recording, "--capture-seconds", "5"))
    assert sum(flow["packets"] for flow in replay["flows"]) == packets
    assert sum(flow["bytes"] for flow in replay["flows"]) == sum(flow["bytes"] for flow in live["flows"])
    print(f"PASS: {version}, screen rendering, real loopback capture ({packets} packets), recording and matching replay")
