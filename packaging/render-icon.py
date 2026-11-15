from pathlib import Path
from PIL import Image
import subprocess

root=Path(__file__).resolve().parents[1]
png=root/"assets"/"liminalnet-symbol.png"
subprocess.run(["rsvg-convert","-w","256","-h","256",str(root/"assets"/"liminalnet-symbol.svg"),"-o",str(png)],check=True)
Image.open(png).save(root/"assets"/"liminalnet.ico",sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
