"""Code-rendered LinkedIn artwork. Requires Pillow; no server or image model.

Run from any directory: python artifacts/social/render_thumbnail.py
Fonts: Geist and Geist Mono, distributed under the accompanying OFL licenses.
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parents[1]
SCALE = 2
WIDTH, HEIGHT = 1200, 630
BACKGROUND = "#0A0A0A"
FOREGROUND = "#EDEDED"
SECONDARY = "#A1A1A1"


def font(size, weight=400, mono=False):
    file = "GeistMono[wght].ttf" if mono else "Geist[wght].ttf"
    face = ImageFont.truetype(str(HERE / "fonts" / file), round(size * SCALE))
    face.set_variation_by_axes([weight])
    return face


def main():
    canvas = Image.new("RGB", (WIDTH * SCALE, HEIGHT * SCALE), BACKGROUND)
    draw = ImageDraw.Draw(canvas)

    def text(x, y, value, size, weight=400, color=FOREGROUND, mono=False):
        face = font(size, weight, mono)
        draw.text((round(x * SCALE), round(y * SCALE)), value, font=face, fill=color, anchor="lt")
        return draw.textlength(value, font=face) / SCALE

    def centered(y, value, size, weight=400, color=FOREGROUND, mono=False):
        length = draw.textlength(value, font=font(size, weight, mono)) / SCALE
        return text((WIDTH - length) / 2, y, value, size, weight, color, mono)

    def right(x, y, value, size, weight=400, color=SECONDARY, mono=False):
        length = draw.textlength(value, font=font(size, weight, mono)) / SCALE
        return text(x - length, y, value, size, weight, color, mono)

    # A common 64 px safe margin; metadata is typography, not boxed badges.
    text(64, 58, "RUST + ICED", 19, 500, SECONDARY, mono=True)
    right(1136, 61, "v0.1.0 / WINDOWS PREVIEW", 15, mono=True)

    # Use the real application mark, with one large wordmark as the focal object.
    name = "rusty_dlp"
    title_font = font(132, 600)
    title_width = draw.textlength(name, font=title_font) / SCALE
    title_box = draw.textbbox((0, 0), name, font=title_font, anchor="lt")
    title_height = (title_box[3] - title_box[1]) / SCALE
    icon_size, gap, center_y = 128, 30, 284
    start_x = (WIDTH - icon_size - gap - title_width) / 2
    icon = Image.open(PROJECT / "assets/branding/icon.icns").convert("RGBA")
    icon = icon.resize((icon_size * SCALE, icon_size * SCALE), Image.Resampling.LANCZOS)
    canvas.paste(icon, (round(start_x * SCALE), round((center_y - icon_size / 2) * SCALE)), icon)
    text(start_x + icon_size + gap, center_y - title_height / 2, name, 132, 600)

    centered(390, "A native desktop GUI for yt-dlp.", 31, 400)
    centered(445, "Playlists. Quality control. Audio conversion.", 23, color=SECONDARY)

    text(64, 562, "github.com/dilukshann7/rusty_dlp", 18, color=SECONDARY, mono=True)
    right(1136, 565, "INSTALLER + PORTABLE", 15, mono=True)

    # 2× export for sharp social uploads; the companion keeps the 1200×630 canvas.
    canvas.save(HERE / "rusty-dlp-linkedin.png", optimize=True, dpi=(144, 144))
    canvas.resize((WIDTH, HEIGHT), Image.Resampling.LANCZOS).save(
        HERE / "rusty-dlp-linkedin-1200.png", optimize=True
    )
    print("Rendered 2400×1260 and 1200×630 PNGs")


if __name__ == "__main__":
    main()
