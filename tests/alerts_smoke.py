"""Trigger live threshold, favorite and CIDR alerts against loopback only."""
import fcntl
import os
import pathlib
import pty
import select
import socket
import struct
import subprocess
import sys
import tempfile
import termios
import time
import re

binary=str(pathlib.Path(sys.argv[1]).resolve())
for unit in ["Bits","Bytes","Packets"]:
    with tempfile.TemporaryDirectory(prefix="liminalnet-alerts-") as folder:
        root=pathlib.Path(folder)
        blacklist=root/"blacklist.txt";blacklist.write_text("127.0.0.0/8\n")
        (root/"settings.toml").write_text(f'welcomed = true\nfrench = false\nthreshold = 1\nthreshold_unit = "{unit}"\nalert_blacklist = true\nalert_favorites = true\nblacklist = "{blacklist}"\nfavorites = ["0:127.0.0.1"]\n')
        master,slave=pty.openpty()
        fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack("HHHH",40,140,0,0))
        output=bytearray()
        def drain(seconds):
            end=time.monotonic()+seconds
            while time.monotonic()<end:
                if select.select([master],[],[],.05)[0]:
                    try:output.extend(os.read(master,65536))
                    except OSError:break
        process=subprocess.Popen([binary,"--interface","lo"],stdin=slave,stdout=slave,stderr=slave,env=dict(os.environ,TERM="xterm-256color",LIMINALNET_CONFIG_DIR=folder))
        try:
            drain(.3);os.write(master,b"\r");drain(.5)
            with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as receiver,socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sender:
                receiver.bind(("127.0.0.1",0))
                for _ in range(20):sender.sendto(b"local alert fixture",receiver.getsockname())
                drain(1.4)
            os.write(master,b"3");drain(.2)
            plain=re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]",b" ",bytes(output))
            for expected in [b"threshold",b"exceeded",b"BLACKLIST",b"FAVORITE"]:assert expected in plain,(unit,expected,plain.decode(errors="replace")[-3000:])
            for keys in [b"q",b"y",b"q"]:os.write(master,keys);drain(.15)
            process.wait(timeout=4)
            assert process.returncode==0
        finally:
            if process.poll() is None:process.terminate();process.wait(timeout=4)
            os.close(master);os.close(slave)
print("PASS: live bits/bytes/packets thresholds, favorite alerts and CIDR blacklist alerts")
