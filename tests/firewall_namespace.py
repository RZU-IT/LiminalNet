"""Run ONLY in a fresh network namespace; never change the host firewall."""
import json
import os
import pathlib
import subprocess
import sys
import tempfile

binary = str(pathlib.Path(sys.argv[1]).resolve())
assert os.geteuid() == 0, "Root is required inside the test namespace"
assert os.readlink("/proc/self/ns/net") != os.readlink("/proc/1/ns/net"), "Refusing to test the host network namespace"

def ping():
    return subprocess.run(["ping", "-c", "1", "-W", "1", "127.0.0.2"], capture_output=True).returncode == 0

subprocess.run(["ip", "link", "set", "lo", "up"], check=True)
assert ping(), "Baseline loopback connectivity failed"
with tempfile.TemporaryDirectory(prefix="liminalnet-firewall-") as folder:
    env = dict(os.environ, XDG_CONFIG_HOME=folder)
    created = subprocess.run([binary, "--block-ip", "127.0.0.2"], env=env, capture_output=True, text=True, check=True)
    rule = json.loads(created.stdout)
    try:
        assert not ping(), "Traffic still succeeds after installing the rule"
    finally:
        subprocess.run([binary, "--unblock", rule["id"]], env=env, check=True)
    assert ping(), "Connectivity was not restored after removal"
print("PASS: isolated Linux IP rule blocks actual traffic and removal restores it")
