"""Render windows/cli/assets/app.ico from the site mark (dark plate, mint ww)."""

from __future__ import annotations

import io
import struct
import sys
import urllib.request
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
OUT_ICO = HERE / "app.ico"
OUT_PNG = HERE / "app.png"

BG = (7, 9, 12, 255)
MINT = (125, 206, 160, 255)
TRANSPARENT = (0, 0, 0, 0)

# 16px: one W — two letters will not read at this size (M-shaped if the valley is filled)
W16 = (
    (1, 0, 0, 0, 0, 0, 0, 1),
    (1, 0, 0, 0, 0, 0, 0, 1),
    (1, 0, 0, 1, 1, 0, 0, 1),
    (1, 0, 1, 0, 0, 1, 0, 1),
    (1, 1, 0, 0, 0, 0, 1, 1),
    (1, 0, 0, 0, 0, 0, 0, 1),
)

# 20px: two 6×6 w glyphs
W20 = (
    (1, 0, 0, 0, 0, 1),
    (1, 0, 0, 0, 0, 1),
    (1, 0, 1, 0, 0, 1),
    (1, 0, 1, 0, 1, 1),
    (0, 1, 0, 1, 0, 0),
    (0, 1, 0, 1, 0, 0),
)

# 24px: two 8×8 w glyphs
W24 = (
    (1, 0, 0, 0, 0, 0, 0, 1),
    (1, 0, 0, 0, 0, 0, 0, 1),
    (1, 0, 0, 0, 0, 0, 0, 1),
    (1, 0, 0, 1, 1, 0, 0, 1),
    (1, 0, 1, 0, 0, 1, 0, 1),
    (1, 0, 1, 0, 0, 1, 0, 1),
    (1, 1, 0, 0, 0, 0, 1, 1),
    (1, 0, 0, 0, 0, 0, 0, 1),
)

FONT_URLS = (
    "https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Medium.ttf",
    "https://github.com/IBM/plex/raw/master/packages/plex-mono/fonts/complete/ttf/IBMPlexMono-Medium.ttf",
)

SIZES = (16, 20, 24, 32, 40, 48, 64, 128, 256)


def load_font_bytes() -> bytes:
    cache = Path.home() / ".cache" / "wwm" / "IBMPlexMono-Medium.ttf"
    if cache.is_file() and cache.stat().st_size > 10_000:
        return cache.read_bytes()
    last_err: Exception | None = None
    for url in FONT_URLS:
        try:
            with urllib.request.urlopen(url, timeout=30) as resp:
                data = resp.read()
            if len(data) > 10_000:
                cache.parent.mkdir(parents=True, exist_ok=True)
                cache.write_bytes(data)
                return data
        except Exception as e:  # noqa: BLE001 — best-effort download
            last_err = e
    consola = Path(r"C:\Windows\Fonts\consolab.ttf")
    if consola.is_file():
        return consola.read_bytes()
    raise RuntimeError(f"could not load a mono font ({last_err})")


def blit_glyph(px, origin: tuple[int, int], glyph: tuple[tuple[int, ...], ...], color) -> None:
    ox, oy = origin
    for y, row in enumerate(glyph):
        for x, bit in enumerate(row):
            if bit:
                px[ox + x, oy + y] = color


def plate(size: int, pad: int, radius: int, border: int) -> Image.Image:
    im = Image.new("RGBA", (size, size), TRANSPARENT)
    draw = ImageDraw.Draw(im)
    draw.rounded_rectangle(
        (pad, pad, size - 1 - pad, size - 1 - pad),
        radius=radius,
        fill=BG,
        outline=MINT,
        width=border,
    )
    return im


def render_pixel(size: int, glyph: tuple[tuple[int, ...], ...], pair: bool) -> Image.Image:
    pad = 0 if size <= 20 else 1
    radius = 3 if size <= 16 else (4 if size <= 20 else 5)
    im = plate(size, pad, radius, 1)
    gap = 1 if size <= 20 else 2
    pair_w = len(glyph[0]) * (2 if pair else 1) + (gap if pair else 0)
    pair_h = len(glyph)
    x0 = (size - pair_w) // 2
    y0 = (size - pair_h) // 2
    px = im.load()
    blit_glyph(px, (x0, y0), glyph, MINT)
    if pair:
        blit_glyph(px, (x0 + len(glyph[0]) + gap, y0), glyph, MINT)
    return im


def plate_metrics(size: int) -> tuple[int, int, int]:
    pad = 0 if size <= 32 else max(1, round(size / 32))
    radius = max(5, round(size * 6 / 32))
    border = 1 if size < 40 else max(2, round(size * 1.5 / 32))
    return pad, radius, border


def fit_font(font_bytes: bytes, target_w: float, cap: int) -> ImageFont.FreeTypeFont:
    dummy = ImageDraw.Draw(Image.new("RGBA", (8, 8)))
    lo, hi = 8, max(9, cap)
    best = hi
    for _ in range(20):
        mid = (lo + hi) // 2
        font = ImageFont.truetype(io.BytesIO(font_bytes), mid)
        bbox = dummy.textbbox((0, 0), "ww", font=font)
        tw = bbox[2] - bbox[0]
        if tw > target_w:
            hi = mid - 1
        else:
            best = mid
            lo = mid + 1
    return ImageFont.truetype(io.BytesIO(font_bytes), best)


def render_font_ww(size: int, font_bytes: bytes) -> Image.Image:
    pad, radius, border = plate_metrics(size)
    im = plate(size, pad, radius, border)
    inner = (size - 2 * pad) - 2 * border
    ratio = 0.86 if size <= 48 else 0.74
    # Crisp plate at native size; supersample only the letters.
    ts = 2 if size < 64 else 4
    font = fit_font(font_bytes, inner * ratio * ts, cap=int(inner * ts))
    layer = Image.new("RGBA", (size * ts, size * ts), TRANSPARENT)
    draw = ImageDraw.Draw(layer)
    bbox = draw.textbbox((0, 0), "ww", font=font)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    cx = (size * ts - tw) / 2 - bbox[0]
    cy = (size * ts - th) / 2 - bbox[1]
    draw.text((cx, cy), "ww", font=font, fill=MINT)
    letters = layer.resize((size, size), Image.Resampling.LANCZOS)
    return Image.alpha_composite(im, letters)


def render(size: int, font_bytes: bytes) -> Image.Image:
    if size <= 16:
        return render_pixel(size, W16, pair=False)
    if size <= 20:
        return render_pixel(size, W20, pair=True)
    if size <= 24:
        return render_pixel(size, W24, pair=True)
    return render_font_ww(size, font_bytes)


def write_ico(path: Path, images: list[Image.Image]) -> None:
    pngs: list[bytes] = []
    for im in images:
        buf = io.BytesIO()
        im.save(buf, format="PNG")
        pngs.append(buf.getvalue())
    count = len(images)
    offset = 6 + 16 * count
    entries = bytearray()
    blob = bytearray()
    for im, png in zip(images, pngs, strict=True):
        w = 0 if im.width >= 256 else im.width
        h = 0 if im.height >= 256 else im.height
        entries += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(png), offset)
        offset += len(png)
        blob += png
    path.write_bytes(struct.pack("<HHH", 0, 1, count) + entries + blob)


def main() -> int:
    font_bytes = load_font_bytes()
    images = [render(s, font_bytes) for s in SIZES]
    write_ico(OUT_ICO, images)
    images[-1].save(OUT_PNG)
    print(f"wrote {OUT_ICO} ({OUT_ICO.stat().st_size} bytes) and {OUT_PNG.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
