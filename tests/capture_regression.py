"""Validate capture imports and failure cases against independently built files."""
import json
import os
import pathlib
import struct
import subprocess
import sys
import tempfile

binary = str(pathlib.Path(sys.argv[1]).resolve())
test_env=dict(os.environ)

def udp_frame():
    ethernet = bytes.fromhex("00112233445566778899aabb0800")
    ip = bytes.fromhex("4500001c00000000401100000a00000108080808")
    udp = struct.pack("!HHHH", 50000, 53, 8, 0)
    return ethernet + ip + udp

def pcap(path, frames, link=1):
    with open(path, "wb") as stream:
        stream.write(struct.pack("<IHHIIII", 0xa1b2c3d4, 2, 4, 0, 0, 65535, link))
        for index, frame in enumerate(frames):
            stream.write(struct.pack("<IIII", 1700000000+index, 0, len(frame), len(frame)))
            stream.write(frame)

def pcapng(path, frames):
    with open(path,"wb") as stream:
        stream.write(struct.pack("<IIIHHqI",0x0a0d0d0a,28,0x1a2b3c4d,1,0,-1,28))
        stream.write(struct.pack("<IIHHII",1,20,1,0,65535,20))
        for index,frame in enumerate(frames):
            timestamp=(1700000000+index)*1000000
            padded=frame+b"\0"*((-len(frame))%4)
            length=32+len(padded)
            stream.write(struct.pack("<IIIIIII",6,length,0,timestamp>>32,timestamp&0xffffffff,len(frame),len(frame))+padded+struct.pack("<I",length))

def run(*args, success=True, env=None):
    result = subprocess.run([binary, *map(str,args)], capture_output=True, text=True, timeout=10, env=test_env if env is None else env)
    assert (result.returncode==0)==success, (args,result.returncode,result.stdout,result.stderr)
    return result

with tempfile.TemporaryDirectory(prefix="liminalnet-capture-") as folder:
    folder=pathlib.Path(folder)
    test_env["LIMINALNET_CONFIG_DIR"]=str(folder/"profile")
    source=folder/"fixture.pcap"
    frames=[udp_frame()]*3
    pcap(source,frames)
    result=json.loads(run("--read",source,"--capture-seconds","5").stdout)
    assert sum(f["packets"] for f in result["flows"])==3
    assert sum(f["bytes"] for f in result["flows"])==126
    assert result["flows"][0]["service"]=="domain"
    ng=folder/"fixture.pcapng";pcapng(ng,frames)
    ng_result=json.loads(run("--read",ng,"--capture-seconds","5").stdout)
    assert sum(f["packets"] for f in ng_result["flows"])==3
    assert sum(f["bytes"] for f in ng_result["flows"])==126
    filtered=json.loads(run("--read",source,"--filter","tcp","--capture-seconds","5").stdout)
    assert not filtered["flows"]
    run("--read",source,"--filter","not valid bpf !!","--capture-seconds","5",success=False)
    corrupt=folder/"corrupt.pcap";corrupt.write_bytes(b"not a capture")
    run("--read",corrupt,"--capture-seconds","5",success=False)
    unsupported=folder/"unsupported.pcap";pcap(unsupported,frames,9999)
    run("--read",unsupported,"--capture-seconds","5",success=False)
    existing=folder/"existing.pcap";existing.write_bytes(b"keep this file")
    run("--read",source,"--record",existing,"--capture-seconds","5",success=False)
    assert existing.read_bytes()==b"keep this file"
    env=dict(os.environ,LIMINALNET_CONFIG_DIR=str(folder/"config"))
    path=pathlib.Path(run("--config-path",env=env).stdout.strip())
    path.parent.mkdir(parents=True,exist_ok=True);path.write_text("invalid = [")
    run("--render","sources",env=env,success=False)
    assert path.read_text()=="invalid = ["
print("PASS: PCAP/PCAPNG totals, service mapping, BPF filtering, invalid BPF, corrupt/unsupported capture, no overwrite, corrupt config retained")
