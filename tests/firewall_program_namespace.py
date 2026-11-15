"""Program blocking; isolated network, dedicated cgroup."""
import http.server
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import uuid
import gzip

binary=str(pathlib.Path(sys.argv[1]).resolve())
assert os.geteuid()==0
assert os.readlink("/proc/self/ns/net")!=os.readlink("/proc/1/ns/net"),"Refusing host network namespace"
if pathlib.Path("/proc/config.gz").exists():
    with gzip.open("/proc/config.gz","rt") as stream:
        if "# CONFIG_NFT_SOCKET is not set" in stream.read():
            print("SKIP: kernel lacks CONFIG_NFT_SOCKET; Linux program blocking cannot be validated here")
            sys.exit(77)
subprocess.run(["ip","link","set","lo","up"],check=True)
group=pathlib.Path("/sys/fs/cgroup")/("liminalnet-test-"+uuid.uuid4().hex)
group.mkdir()
child=None
rule=None
with tempfile.TemporaryDirectory(prefix="liminalnet-program-") as folder:
    folder=pathlib.Path(folder)
    executable=folder/"curl-probe"
    shutil.copy2(shutil.which("curl"),executable)
    class Handler(http.server.BaseHTTPRequestHandler):
        count=0
        def do_GET(self):
            time.sleep(.1)
            try:
                self.send_response(200);self.send_header("Content-Length","3");self.end_headers();self.wfile.write(b"ok\n")
                Handler.count+=1
            except (BrokenPipeError,ConnectionResetError):
                pass
        def log_message(self,*args):pass
    server=http.server.ThreadingHTTPServer(("127.0.0.1",0),Handler)
    server.daemon_threads=True
    threading.Thread(target=server.serve_forever,daemon=True).start()
    env=dict(os.environ,XDG_CONFIG_HOME=str(folder/"config"))
    def assign_group():
        (group/"cgroup.procs").write_text(str(os.getpid()))
    try:
        url=f"http://127.0.0.1:{server.server_port}/"
        child=subprocess.Popen([str(executable),"-s","--max-time","1","--connect-timeout","1",*([url]*250)],preexec_fn=assign_group,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        time.sleep(.7)
        assert Handler.count>0,"Baseline client requests failed"
        created=subprocess.run([binary,"--block-program",str(executable)],env=env,capture_output=True,text=True)
        assert created.returncode==0,created.stderr
        rule=json.loads(created.stdout)
        time.sleep(1.5)
        before=Handler.count
        time.sleep(.6)
        assert Handler.count==before,"Requests still succeed after program blocking"
        subprocess.run([binary,"--unblock",rule["id"]],env=env,check=True)
        rule=None
        time.sleep(1.5)
        assert Handler.count>before,"Requests did not resume after rule removal"
        print("PASS: dedicated Linux cgroup program blocking and restoration against a local HTTP server")
    finally:
        if rule is not None:
            subprocess.run([binary,"--unblock",rule["id"]],env=env,capture_output=True)
        if child is not None:
            child.terminate();child.wait(timeout=5)
        server.shutdown();server.server_close()
        group.rmdir()
