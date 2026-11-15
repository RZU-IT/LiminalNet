from pathlib import Path
import tarfile
import tomllib

root=Path(__file__).resolve().parents[1]
folder=root/"dist"/"linux"
executables={"liminalnet","liminalnet-core","install.sh"}
version=tomllib.loads((root/"Cargo.toml").read_text())["package"]["version"]
with tarfile.open(root/"dist"/f"LiminalNet-{version}-linux-x86_64.tar.gz","w:gz") as archive:
    for path in sorted(folder.iterdir()):
        info=archive.gettarinfo(str(path),arcname=path.name)
        info.mode=0o755 if path.name in executables else 0o644
        info.uid=info.gid=0
        info.uname=info.gname="root"
        with path.open("rb") as stream:archive.addfile(info,stream)
