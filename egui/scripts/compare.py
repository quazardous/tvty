#!/usr/bin/env python3
"""Put tvty's screenshot and tvty-egui's side by side, marked, for an issue.

Usage:
    compare.py REF EGUI OUT [--crop X,Y,W,H] [--zoom N]
               [--mark X,Y,W,H,LABEL ...] [--title TEXT]

- REF is the GPUI window, EGUI the egui-pop one, both of the same size.
- --crop keeps a region (window coordinates), --zoom enlarges it (nearest
  neighbour: pixels stay pixels).
- each --mark frames a region (window coordinates) in red on both sides,
  with its label; the labels are listed under the images.
- prints how many pixels differ in the region, and writes OUT.
"""

import argparse

from PIL import Image, ImageChops, ImageDraw, ImageFont

RED = (255, 48, 48)
HEADER = 28
FOOTER_LINE = 22
GAP = 12
BG = (32, 32, 36)
FG = (230, 230, 230)


def font(size):
    for path in ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                 "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"):
        try:
            return ImageFont.truetype(path, size)
        except OSError:
            continue
    return ImageFont.load_default()


def rect(text):
    x, y, w, h = (int(v) for v in text.split(",")[:4])
    return x, y, w, h


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ref")
    ap.add_argument("egui")
    ap.add_argument("out")
    ap.add_argument("--crop")
    ap.add_argument("--zoom", type=int, default=1)
    ap.add_argument("--mark", action="append", default=[])
    ap.add_argument("--title", default="")
    a = ap.parse_args()

    ref = Image.open(a.ref).convert("RGB")
    egui = Image.open(a.egui).convert("RGB")
    cx, cy, cw, ch = rect(a.crop) if a.crop else (0, 0, ref.width, ref.height)
    box = (cx, cy, cx + cw, cy + ch)
    r, e = ref.crop(box), egui.crop(box)
    diff = ImageChops.difference(r, e).convert("L").point(lambda v: 255 if v > 8 else 0)
    changed = sum(diff.histogram()[1:])
    z = max(1, a.zoom)
    r = r.resize((cw * z, ch * z), Image.NEAREST)
    e = e.resize((cw * z, ch * z), Image.NEAREST)

    marks = []
    for m in a.mark:
        parts = m.split(",", 4)
        x, y, w, h = (int(v) for v in parts[:4])
        marks.append(((x - cx) * z, (y - cy) * z, w * z, h * z, parts[4] if len(parts) > 4 else ""))

    small = font(16)
    lines = [f"{i + 1}. {label}" for i, (*_, label) in enumerate(marks) if label]
    width = cw * z * 2 + GAP
    height = HEADER + ch * z + GAP + FOOTER_LINE * (len(lines) + 1)
    out = Image.new("RGB", (width, height), BG)
    draw = ImageDraw.Draw(out)
    draw.text((6, 5), f"tvty (GPUI)  {a.title}", fill=FG, font=small)
    draw.text((cw * z + GAP + 6, 5), "tvty-egui (egui-pop)", fill=FG, font=small)
    for i, (img, left) in enumerate(((r, 0), (e, cw * z + GAP))):
        out.paste(img, (left, HEADER))
        for n, (x, y, w, h, _) in enumerate(marks):
            x0, y0 = left + x, HEADER + y
            draw.rectangle((x0, y0, x0 + w - 1, y0 + h - 1), outline=RED, width=2)
            tag = str(n + 1)
            draw.rectangle((x0, y0, x0 + 8 + 9 * len(tag), y0 + 18), fill=RED)
            draw.text((x0 + 4, y0), tag, fill=(255, 255, 255), font=small)
    y = HEADER + ch * z + GAP
    total = cw * ch
    draw.text((6, y), f"{changed} of {total} pixels differ ({100 * changed / total:.1f} %), region {cx},{cy} {cw}x{ch}",
              fill=FG, font=small)
    for line in lines:
        y += FOOTER_LINE
        draw.text((6, y), line, fill=FG, font=small)
    out.save(a.out)
    print(f"{a.out}: {changed}/{total} pixels differ ({100 * changed / total:.1f} %)")


if __name__ == "__main__":
    main()
