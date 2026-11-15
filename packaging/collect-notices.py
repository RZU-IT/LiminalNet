"""Collect dependency license texts from Cargo's resolved local sources."""
import json
import pathlib
import subprocess

root=pathlib.Path(__file__).resolve().parents[1]
metadata=json.loads(subprocess.check_output([str(pathlib.Path.home()/".cargo/bin/cargo"),"metadata","--locked","--format-version","1"],cwd=root))
parts=["LiminalNet third-party dependency notices\nEach component retains its own license.\n"]
for package in sorted(metadata["packages"],key=lambda p:(p["name"],p["version"])):
    if package["name"]=="liminalnet":continue
    folder=pathlib.Path(package["manifest_path"]).parent
    parts.append(f"\n{'='*72}\n{package['name']} {package['version']}\nLicense expression: {package.get('license')}\nRepository: {package.get('repository')}\n")
    files=[]
    if package.get("license_file"):
        path=folder/package["license_file"]
        if path.is_file():files.append(path)
    for path in folder.iterdir():
        if path.is_file() and path.name.upper().startswith(("LICENSE","COPYING","COPYRIGHT")) and path not in files:files.append(path)
    for path in sorted(files):parts.append(f"\n--- {path.name} ---\n"+path.read_text(errors="replace"))
(root/"LICENSES"/"THIRD-PARTY-NOTICES.txt").write_text("\n".join(parts),encoding="utf-8")
print(f"Collected notices for {len(metadata['packages'])-1} resolved dependency packages")
