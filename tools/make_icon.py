#!/usr/bin/env python3
"""Draws SunnyF3's icon.png (128x128): a 32x32 pixel picture, each pixel 4x4.
A sun over a dark debug panel with coloured lines of text."""
import math, struct, sys, zlib

G = 32
px = [[(0, 0, 0, 0)] * G for _ in range(G)]

def put(x, y, c):
    if 0 <= x < G and 0 <= y < G:
        px[y][x] = c if len(c) == 4 else (*c, 255)

def rect(x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(x, y, c)

def rounded(x0, y0, x1, y1, c, r=1, bg=(0, 0, 0, 0)):
    rect(x0, y0, x1, y1, c)
    for (cx, cy) in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)):
        for i in range(r):
            for j in range(r - i):
                put(cx + (i if cx == x0 else -i), cy + (j if cy == y0 else -j), bg)

NAVY = (24, 27, 42)
rounded(0, 0, 31, 31, (30, 34, 52), 3)
rounded(1, 1, 30, 30, NAVY, 2, (30, 34, 52, 255))

# "F3" in a chunky pixel font, warm gradient, top right.
F = ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]
T = ["####.", "....#", "....#", ".###.", "....#", "....#", "####."]
def glyph(g, ox, oy):
    for j, row in enumerate(g):
        for i, ch in enumerate(row):
            if ch == "#":
                t = j / 6
                c = (255, int(214 - 90 * t), int(90 - 60 * t))
                put(ox + i, oy + j, (120, 50, 20))      # shadow, under and right
                put(ox + i + 1, oy + j + 1, (120, 50, 20))
    for j, row in enumerate(g):
        for i, ch in enumerate(row):
            if ch == "#":
                t = j / 6
                put(ox + i, oy + j, (255, int(214 - 90 * t), int(90 - 60 * t)))
glyph(F, 16, 5)
glyph(T, 23, 5)

# The sun, top left.
cx, cy = 8.5, 8.5
for y in range(G):
    for x in range(G):
        dx, dy = x + 0.5 - cx, y + 0.5 - cy
        r = math.hypot(dx, dy)
        a = math.degrees(math.atan2(dy, dx)) % 45
        if r <= 3.6:
            put(x, y, (254, 216, 61) if r <= 2.6 else (249, 128, 29))
            if r <= 1.5 and dx < 0 and dy < 0:
                put(x, y, (255, 244, 180))
        elif 4.6 <= r <= 6.4 and min(a, 45 - a) < 7:
            put(x, y, (249, 160, 40))

# Debug lines: a grey key, then a coloured value, each on a darker strip.
STRIP = (14, 16, 26)
rows = [
    (18, 4, (110, 224, 110), 9),   # FPS: green
    (22, 6, (255, 122, 122), 5),   # X
    (26, 5, (122, 180, 255), 11),  # Z / blue
]
for (y, key_w, colour, val_w) in rows:
    x0 = 4
    rect(x0 - 1, y - 1, x0 + key_w + val_w + 2, y + 2, STRIP)
    rect(x0, y, x0 + key_w - 1, y + 1, (168, 173, 184))
    rect(x0 + key_w + 1, y, x0 + key_w + val_w, y + 1, colour)
# A thin accent edge down the left, like SunnyF3's panels.
rect(2, 17, 2, 28, (249, 128, 29))

out = []
N = 128
for y in range(N):
    row = bytearray([0])
    for x in range(N):
        row += bytes(px[y // 4][x // 4])
    out.append(bytes(row))
raw = b"".join(out)
def chunk(t, d):
    return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", N, N, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
open(sys.argv[1] if len(sys.argv) > 1 else "icon.png", "wb").write(png)
