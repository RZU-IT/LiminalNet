"""Terminal lifecycle with isolated settings."""
import fcntl
import os
import pathlib
import pty
import select
import struct
import subprocess
import sys
import tempfile
import re
import termios
import time

binary = pathlib.Path(sys.argv[1]).resolve()
capture_file = pathlib.Path(sys.argv[2]).resolve() if len(sys.argv)>2 else None
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 140, 0, 0))
before = termios.tcgetattr(slave)
output = bytearray()

def drain(seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if select.select([master], [], [], 0.05)[0]:
            try:
                output.extend(os.read(master, 65536))
            except OSError:
                break

with tempfile.TemporaryDirectory(prefix="liminalnet-smoke-") as folder:
    env = dict(os.environ, TERM="xterm-256color", LIMINALNET_CONFIG_DIR=folder)
    arguments=[str(binary),"--interface","lo"]
    if capture_file:arguments.extend(["--read",str(capture_file)])
    process = subprocess.Popen(arguments, stdin=slave, stdout=slave, stderr=slave, env=env)
    try:
        drain(0.7)
        sequence=[(b"\r", .3), (b"\r", .6), (b" ", .3), (b" ", .3), (b"2", .2), (b"s", .2), (b"t", .2), (b"\x1b", .2), (b"q", .2), (b"y", .4), (b"q", .2)]
        if capture_file:sequence=[(b"\r",.3),(b"\r",.6),(b"2",.2),(b"\r",.2),(b"p",2.5),(b"\x1b",.2),(b"q",.2),(b"y",.3),(b"q",.2)]
        for keys, delay in sequence:
            os.write(master, keys)
            drain(delay)
        process.wait(timeout=5)
        drain(.1)
        assert process.returncode == 0, output.decode(errors="replace")[-3000:]
        assert termios.tcgetattr(slave) == before, "Terminal mode was not restored"
        assert b"Welcome to the Threshold" in output
        assert b"panic" not in output.lower()
        assert b"Cannot open capture" not in output, "Capture permissions missing; rerun this test with capture privileges"
        if capture_file:
            assert b"MAC:" in output,"Connection details were not shown"
            plain=re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]",b" ",bytes(output))
            assert b"rtt" in plain and b"mdev" in plain,("Local RTT probe did not complete",plain.decode(errors="replace")[-4000:])
            print("PASS: capture import, connection details, explicit local RTT probe, stop confirmation, terminal restoration")
        else:
            assert b"CAPTURE ACTIVE" in output, "Active capture was not shown"
            assert b"EN PAUSE" in output, "Pause acknowledgement was not shown"
            print("PASS: welcome, sources, capture, pause/resume, tabs, settings, stop confirmation, quit, terminal restoration")
    finally:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)
