#!/usr/bin/env python3
"""The pictures the demo's tickets carry: blueprints and charts, drawn here.

Usage: images.py OUT_DIR — writes one PNG per picture there.
"""

import math
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

W, H = 720, 420
PAPER = (18, 42, 74)
GRID = (30, 64, 104)
INK = (190, 220, 255)
DIM = (110, 150, 200)
HOT = (255, 120, 80)
OK = (120, 230, 160)


def _monospace():
    """The system's monospace font, as fontconfig names it."""
    try:
        path = subprocess.run(["fc-match", "-f", "%{file}", "monospace"], capture_output=True, text=True).stdout
    except OSError:
        return None
    return path if path and Path(path).exists() else None


MONOSPACE = _monospace()


def font(size):
    if MONOSPACE:
        return ImageFont.truetype(MONOSPACE, size)
    return ImageFont.load_default(size)


def paper(title):
    image = Image.new("RGB", (W, H), PAPER)
    d = ImageDraw.Draw(image)
    for x in range(0, W, 24):
        d.line([(x, 0), (x, H)], fill=GRID)
    for y in range(0, H, 24):
        d.line([(0, y), (W, y)], fill=GRID)
    d.rectangle([8, 8, W - 9, H - 9], outline=DIM)
    d.text((20, 16), title, fill=INK, font=font(18))
    return image, d


def exhaust_port(out):
    image, d = paper("STATION — THERMAL EXHAUST PORT · sheet 7 of 7")
    cx, cy, r = 250, 225, 160
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=INK, width=3)
    # The dish, and the trench round the equator.
    d.ellipse([cx - 95, cy - 110, cx - 5, cy - 20], outline=INK, width=2)
    d.ellipse([cx - 62, cy - 77, cx - 38, cy - 53], outline=DIM, width=2)
    d.line([(cx - r, cy + 4), (cx + r, cy + 4)], fill=INK, width=2)
    d.line([(cx - r + 2, cy + 14), (cx + r - 2, cy + 14)], fill=DIM, width=1)
    # The port, at the end of the trench.
    px, py = cx + 118, cy + 9
    d.ellipse([px - 7, py - 7, px + 7, py + 7], outline=HOT, width=3)
    d.line([(px + 10, py - 10), (465, 120)], fill=HOT, width=2)
    d.text((470, 100), "EXHAUST PORT", fill=HOT, font=font(18))
    d.text((470, 126), "Ø 2 m", fill=INK, font=font(16))
    d.text((470, 150), "→ main reactor", fill=INK, font=font(16))
    d.text((470, 174), "grate: none", fill=HOT, font=font(16))
    d.text((470, 290), "ray shield   on", fill=OK, font=font(16))
    d.text((470, 314), "particle     OFF", fill=HOT, font=font(16))
    d.text((470, 338), "turbolasers  on", fill=OK, font=font(16))
    image.save(out / "exhaust-port.png")


def superlaser(out):
    image, d = paper("SUPERLASER — CHARGE TIME, h (lower is better)")
    left, bottom, top, right = 80, 360, 80, 660
    d.line([(left, top), (left, bottom), (right, bottom)], fill=INK, width=2)
    runs = [("v1", 96), ("v2", 72), ("v3", 51), ("v4", 38), ("plan", 23)]
    bar = (right - left) // len(runs)
    for i, (label, hours) in enumerate(runs):
        x = left + 20 + i * bar
        height = (bottom - top) * hours / 100
        colour = OK if label == "plan" else INK
        d.rectangle([x, bottom - height, x + bar - 40, bottom], outline=colour, width=2)
        d.text((x + 4, bottom - height - 24), f"{hours} h", fill=colour, font=font(16))
        d.text((x + 8, bottom + 8), label, fill=DIM, font=font(16))
    target = bottom - (bottom - top) * 24 / 100
    d.line([(left, target), (right, target)], fill=HOT, width=1)
    d.text((left + 8, target - 22), "target 24 h", fill=HOT, font=font(15))
    image.save(out / "superlaser.png")


def compactor(out):
    image, d = paper("WASTE COMPACTOR 3263827 — LIFE SENSOR")
    d.rectangle([140, 110, 580, 330], outline=INK, width=3)
    d.line([(160, 110), (160, 330)], fill=DIM, width=6)
    d.line([(560, 110), (560, 330)], fill=DIM, width=6)
    d.text((70, 210), "wall →", fill=DIM, font=font(16))
    d.text((590, 210), "← wall", fill=DIM, font=font(16))
    for x in range(200, 540, 36):
        d.arc([x, 290, x + 30, 320], 180, 360, fill=DIM, width=2)
    d.ellipse([340, 190, 380, 230], outline=OK, width=3)
    d.text((300, 240), "sensor", fill=OK, font=font(16))
    d.text((170, 350), "something alive inside → walls stop", fill=OK, font=font(17))
    image.save(out / "compactor.png")


def sphere(out):
    image, d = paper("DYSON SPHERE — PROGRESS")
    cx, cy = 300, 225
    d.ellipse([cx - 40, cy - 40, cx + 40, cy + 40], fill=(255, 210, 120), outline=HOT, width=2)
    d.ellipse([cx - 160, cy - 160, cx + 160, cy + 160], outline=DIM, width=1)
    for i in range(14):
        a = math.radians(200 + i * 4.2)
        x, y = cx + 160 * math.cos(a), cy + 160 * math.sin(a)
        d.rectangle([x - 5, y - 5, x + 5, y + 5], fill=OK)
    d.text((500, 150), "panels bolted", fill=INK, font=font(16))
    d.text((500, 174), "4 812", fill=OK, font=font(26))
    d.text((500, 220), "to go", fill=INK, font=font(16))
    d.text((500, 244), "999 995 188", fill=HOT, font=font(26))
    d.text((500, 300), "ETA: 41 017 years", fill=DIM, font=font(16))
    image.save(out / "dyson-sphere.png")


def timeline(out):
    image, d = paper("TIME MACHINE — WHERE THE BUG IS")
    y = 220
    d.line([(60, y), (660, y)], fill=INK, width=3)
    for x, label in ((120, "mon"), (260, "tue"), (400, "wed"), (540, "thu")):
        d.line([(x, y - 8), (x, y + 8)], fill=INK, width=2)
        d.text((x - 14, y + 16), label, fill=DIM, font=font(16))
    # Thursday's fix, shipped on Monday.
    d.arc([120, 90, 540, 350], 180, 360, fill=HOT, width=3)
    d.polygon([(120, 220), (112, 204), (130, 206)], fill=HOT)
    d.text((250, 70), "fix committed thu → merged mon", fill=HOT, font=font(17))
    d.text((170, 300), "the test fails before the bug exists", fill=INK, font=font(17))
    image.save(out / "timeline.png")


if __name__ == "__main__":
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for draw in (exhaust_port, superlaser, compactor, sphere, timeline):
        draw(out)
    print(" ".join(sorted(p.name for p in out.glob("*.png"))))
