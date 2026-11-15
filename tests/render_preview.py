"""Render the application's actual terminal cells for visual review."""
import json
import pathlib
import sys
from PIL import Image, ImageDraw, ImageFont

data=json.loads(pathlib.Path(sys.argv[1]).read_text())
font=ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",15)
width,height=10,20
image=Image.new("RGB",(data["width"]*width,data["height"]*height))
draw=ImageDraw.Draw(image)
for cell in data["cells"]:
    x,y=cell["x"]*width,cell["y"]*height
    draw.rectangle((x,y,x+width-1,y+height-1),fill=cell["bg"])
for cell in data["cells"]:
    text=cell["text"]
    if len(text)==1 and 0x2800<=ord(text)<=0x28ff:
        bits=ord(text)-0x2800
        for bit,(column,row) in enumerate([(0,0),(0,1),(0,2),(1,0),(1,1),(1,2),(0,3),(1,3)]):
            if bits & (1<<bit):
                x,y=cell["x"]*width+2+column*5,cell["y"]*height+2+row*4
                draw.ellipse((x,y,x+1,y+1),fill=cell["fg"])
    else:
        draw.text((cell["x"]*width,cell["y"]*height),text,font=font,fill=cell["fg"])
image.save(sys.argv[2])
