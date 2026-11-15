"""Autostart registration in isolated folders."""
import os
import pathlib
import subprocess
import sys
import tempfile

binary=str(pathlib.Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix="liminalnet-autostart-") as folder:
    root=pathlib.Path(folder)
    env=dict(os.environ,LIMINALNET_CONFIG_DIR=str(root/"profile"),XDG_CONFIG_HOME=str(root/"xdg"),APPDATA=str(root/"appdata"))
    if os.name=="nt":
        registration=root/"appdata/Microsoft/Windows/Start Menu/Programs/Startup/LiminalNet.lnk"
    else:
        registration=root/"xdg/autostart/liminalnet.desktop"
    subprocess.run([binary,"--autostart","true"],env=env,check=True,capture_output=True)
    assert registration.is_file(),"Startup registration missing"
    subprocess.run([binary,"--autostart","false"],env=env,check=True,capture_output=True)
    assert not registration.exists(),"Startup registration was not removed"
print("PASS: autostart registration and removal in isolated user folders")
