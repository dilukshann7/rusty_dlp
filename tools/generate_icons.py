"""Export the checked-in geometric SVG mark to PNG, ICO and ICNS (Pillow)."""
from pathlib import Path
import xml.etree.ElementTree as ET
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]


def main():
    folder = ROOT / "assets" / "branding"
    svg = ET.parse(folder / "logo.svg").getroot()
    scale = 16
    image = Image.new("RGBA", (64 * scale, 64 * scale))
    draw = ImageDraw.Draw(image)
    for element in svg:
        kind = element.tag.rsplit("}", 1)[-1]
        if kind == "rect":
            x, y, width, height = [float(element.attrib[k]) * scale for k in ("x", "y", "width", "height")]
            draw.rounded_rectangle((x, y, x + width, y + height), radius=float(element.get("rx", 0)) * scale,
                                   fill=element.get("fill"), outline=element.get("stroke"),
                                   width=max(1, round(float(element.get("stroke-width", 0)) * scale)))
        elif kind == "polygon":
            points = [tuple(float(v) * scale for v in point.split(",")) for point in element.attrib["points"].split()]
            draw.polygon(points, fill=element.get("fill"))
        else:
            raise ValueError(f"Unsupported SVG element: {kind}")
    image.resize((256, 256), Image.Resampling.LANCZOS).save(folder / "icon.png")
    image.save(folder / "icon.ico", sizes=[(n, n) for n in (16, 24, 32, 48, 64, 128, 256)])
    image.save(folder / "icon.icns")
    raster = Image.open(folder / "icon.png").convert("RGBA")
    small = Image.new("RGB", (55, 55), "white")
    mark = raster.resize((49, 49), Image.Resampling.LANCZOS)
    small.paste(mark, (3, 3), mark)
    small.save(folder / "wizard-small.bmp")
    banner = Image.new("RGB", (164, 314), "#0A0A0A")
    mark = raster.resize((112, 112), Image.Resampling.LANCZOS)
    banner.paste(mark, (26, 101), mark)
    banner.save(folder / "wizard-large.bmp")
    print("Exported PNG, seven-size ICO, ICNS, and installer artwork from logo.svg")


if __name__ == "__main__":
    main()
