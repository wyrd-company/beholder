#!/usr/bin/env python3
"""Generate the beholder brand SVGs into docs/user/assets/brand.

The mark geometry is defined once here. Every variant (light, dark, favicon,
lockup, social preview) is emitted from it so that the variants cannot drift.

Run: python3 scripts/brand.py   (then scripts/brand-png.sh for raster files)
"""

from __future__ import annotations

import math
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "docs" / "user" / "assets" / "brand"

INK_LIGHT = "#16181d"  # ink on light backgrounds
INK_DARK = "#e9ebf0"  # ink on dark backgrounds
BG_DARK = "#0d1117"

C = (256.0, 256.0)
PUPIL_R = 46
IRIS_R = 86
IRIS_W = 16
LID_W = 10
TRACE_W = 8
SMALL_IRIS_R = 140
SMALL_IRIS_W = 44
SMALL_PUPIL_K = 1.65


def f(v: float) -> str:
    s = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if s == "-0" else s


# --- geometry helpers -------------------------------------------------------


def cubic(p0, p1, p2, p3, t):
    u = 1 - t
    return tuple(
        u**3 * a + 3 * u * u * t * b + 3 * u * t * t * c + t**3 * d
        for a, b, c, d in zip(p0, p1, p2, p3)
    )


def lid(apex_y: float):
    """A lid is two mirrored cubics from the left corner to the right corner."""
    a, b = (44.0, 256.0), (468.0, 256.0)
    k = apex_y
    first = (a, (104.0, 256 + (k - 256) * 0.72), (186.0, k), (256.0, k))
    second = ((256.0, k), (326.0, k), (408.0, 256 + (k - 256) * 0.72), b)

    def at(t: float):
        if t <= 0.5:
            return cubic(*first, t * 2)
        return cubic(*second, (t - 0.5) * 2)

    return at


UPPER = lid(126)
LOWER = lid(386)


def polyline_on(curve, t0: float, t1: float, n: int = 24) -> str:
    pts = [curve(t0 + (t1 - t0) * i / n) for i in range(n + 1)]
    return "M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts)


def arc(r: float, a0: float, a1: float) -> str:
    """Clockwise arc around C from angle a0 to a1 (degrees, y down)."""
    x0, y0 = C[0] + r * math.cos(math.radians(a0)), C[1] + r * math.sin(math.radians(a0))
    x1, y1 = C[0] + r * math.cos(math.radians(a1)), C[1] + r * math.sin(math.radians(a1))
    large = 1 if (a1 - a0) % 360 > 180 else 0
    return f"M{f(x0)} {f(y0)} A{f(r)} {f(r)} 0 {large} 1 {f(x1)} {f(y1)}"


def ring_point(angle: float, r: float = IRIS_R + IRIS_W / 2):
    return (C[0] + r * math.cos(math.radians(angle)), C[1] + r * math.sin(math.radians(angle)))


def pad(x: float, y: float, s: float) -> str:
    rx = max(2.0, s * 0.16)
    return f'<rect x="{f(x - s / 2)}" y="{f(y - s / 2)}" width="{f(s)}" height="{f(s)}" rx="{f(rx)}"/>'


def line(*pts) -> str:
    return "M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts)


# --- the mark ---------------------------------------------------------------


def mark_parts(detail: str = "full"):
    """Return (stroked paths [(d, width)], filled shapes [svg], pupil svg)."""
    strokes: list[tuple[str, float]] = []
    fills: list[str] = []

    # Iris ring, broken where traces meet it.
    if detail == "full":
        ring_r, ring_w, gaps = IRIS_R, IRIS_W, [(-84, -66), (14, 32), (82, 98), (166, 194)]
    else:
        ring_r, ring_w, gaps = SMALL_IRIS_R, SMALL_IRIS_W, [(-14, 14), (166, 194)]
    for i, (_, end) in enumerate(gaps):
        nxt = gaps[(i + 1) % len(gaps)][0] + (360 if i + 1 == len(gaps) else 0)
        strokes.append((arc(ring_r, end, nxt), ring_w))

    if detail == "full":
        # Upper lid segments, broken at pads and crossings.
        for t0, t1 in [(0.195, 0.5), (0.545, 0.64), (0.69, 0.77)]:
            strokes.append((polyline_on(UPPER, t0, t1), LID_W))
        for t0, t1 in [(0.0, 0.47), (0.53, 0.84), (0.9, 0.97)]:
            strokes.append((polyline_on(LOWER, t0, t1), LID_W))

        ul = UPPER(0.147)  # 45 degrees up-left of the iris pad
        ur = UPPER(0.81)
        pi = (C[0] - IRIS_R, C[1])  # pad seated in the iris' left gap

        # Left: axis trace from the big pad through the lid corner to the iris.
        fills.append(pad(26, 256, 36))
        strokes.append((line((44, 256), (pi[0] - 12, 256)), TRACE_W))
        fills.append(pad(*pi, 26))
        fills.append(pad(*ul, 26))
        strokes.append((line((ul[0] + 8, ul[1] + 8), (pi[0] - 8, pi[1] - 8)), TRACE_W))
        fills.append(pad(76, 226, 18))
        strokes.append((line((76, 235), (76, 256)), TRACE_W))
        fills.append(pad(58, 164, 14))
        lx = 138.0
        ly = min((LOWER(t / 400) for t in range(400)), key=lambda p: abs(p[0] - lx))[1]
        strokes.append((line((lx, ly), (lx, 372)), TRACE_W))
        fills.append(pad(lx, 380, 24))
        for y in (178, 196):
            fills.append(pad(186, y, 8))

        # Top: trace from the top pad down into the iris' upper gap.
        tx = ring_point(-75)[0]
        fills.append(pad(tx, 62, 32))
        strokes.append((line((tx, 78), ring_point(-75)), TRACE_W))
        fills.append(pad(tx, 20, 16))
        fills.append(pad(352, 84, 20))
        ux = 336.0
        uy = min((UPPER(t / 400) for t in range(400)), key=lambda p: abs(p[0] - ux))[1]
        strokes.append((line((346, 94), (ux, 104), (ux, uy)), TRACE_W))

        # Right: lid pad fans out to the iris, a corner pad and the big pad.
        fills.append(pad(*ur, 26))
        rp = ring_point(-28)
        strokes.append((line((ur[0] - 8, ur[1] + 8), rp), TRACE_W))
        fills.append(pad(ur[0] + 42, ur[1] - 42, 18))
        strokes.append((line((ur[0] + 8, ur[1] - 8), (ur[0] + 34, ur[1] - 34)), TRACE_W))
        fills.append(pad(484, 276, 34))
        strokes.append((line((ur[0] + 8, ur[1] + 8), (468, 258)), TRACE_W))
        q = ring_point(23, IRIS_R + 30)
        fills.append(pad(*q, 26))
        strokes.append((line((q[0] + 13, q[1]), (436, q[1]), (450, q[1] - 14), (467, q[1] - 14)), TRACE_W))
        for y in (206, 224, 242):
            fills.append(pad(446, y, 8))
        fills.append(pad(436, 330, 12))

        # Bottom: trace from the iris' lower gap to the bottom pad.
        bx = ring_point(90)[0]
        fills.append(pad(bx, 438, 30))
        strokes.append((line(ring_point(90), (bx, 423)), TRACE_W))
        fills.append(pad(292, 420, 12))
        strokes.append((line((bx + 15, 430), (284, 430), (286, 420)), TRACE_W))
        fills.append(pad(bx, 474, 8))
        fills.append(pad(bx, 492, 8))
        fills.append(pad(290, 452, 8))
    else:
        # Small sizes: the iris, the pupil and the axis traces to two pads.
        # Lids and minor pads do not survive below ~64px, so they are dropped.
        fills.append(pad(52, 256, 88))
        fills.append(pad(460, 256, 88))
        strokes.append((line((96, 256), (C[0] - SMALL_IRIS_R + 6, 256)), 32))
        strokes.append((line((C[0] + SMALL_IRIS_R - 6, 256), (416, 256)), 32))

    return strokes, fills


def pupil(color: str, uid: str, k: float = 1.0) -> str:
    hx, hy, hr = C[0] + 22 * k, C[1] - 24 * k, 16 * k
    return (
        f'<mask id="{uid}"><rect width="512" height="512" fill="#fff"/>'
        f'<circle cx="{f(hx)}" cy="{f(hy)}" r="{f(hr)}" fill="#000"/></mask>'
        f'<circle cx="{f(C[0])}" cy="{f(C[1])}" r="{f(PUPIL_R * k)}" fill="{color}" mask="url(#{uid})"/>'
    )


def mark_group(color: str, detail: str = "full", uid: str = "beholder-pupil", scale_strokes: float = 1.0) -> str:
    strokes, fills = mark_parts(detail)
    w = scale_strokes
    paths = "".join(f'<path d="{d}" stroke-width="{f(sw * w)}"/>' for d, sw in strokes)
    return (
        f'<g fill="none" stroke="{color}" stroke-linecap="butt" stroke-linejoin="round">{paths}</g>'
        f'<g fill="{color}">{"".join(fills)}</g>'
        + pupil(color, uid, 1.0 if detail == "full" else SMALL_PUPIL_K)
    )


# --- wordmark ---------------------------------------------------------------
# Monoline glyphs on a 40 unit x-height with a 64 unit ascender, drawn in the
# same stroke vocabulary as the traces. Baseline is y=0.

XH, ASC, R = 40, 66, 20


def circle_d(cx, cy, r):
    return f"M{f(cx - r)} {f(cy)} a{f(r)} {f(r)} 0 1 0 {f(2 * r)} 0 a{f(r)} {f(r)} 0 1 0 {f(-2 * r)} 0"


def glyph(ch: str, x: float):
    """Return (stroke d list, fill svg list, advance)."""
    o = circle_d(x + R, -R, R)
    if ch == "b":
        return [f"M{f(x)} {-ASC} V0", o], [], 2 * R
    if ch == "d":
        return [f"M{f(x + 2 * R)} {-ASC} V0", o], [], 2 * R
    if ch == "o":
        return [o], [], 2 * R
    if ch == "h":
        return [f"M{f(x)} {-ASC} V0", f"M{f(x)} {-R} A{R} {R} 0 0 1 {f(x + 2 * R)} {-R} V0"], [], 2 * R
    if ch == "l":
        return [f"M{f(x)} {-ASC} V0"], [], 0
    if ch == "e":
        a = math.radians(40)
        ex, ey = x + R + R * math.cos(a), -R + R * math.sin(a)
        return [f"M{f(x)} {-R} H{f(x + 2 * R)} A{R} {R} 0 1 0 {f(ex)} {f(ey)}"], [], 2 * R
    if ch == "r":
        return [f"M{f(x)} {-XH} V0", f"M{f(x)} {-R} A{R} {R} 0 0 1 {f(x + R)} {-XH} H{f(x + R + 6)}"], [
            pad(x + R + 13, -XH, 14)
        ], R + 20
    raise ValueError(ch)


def wordmark(color: str, x0: float = 0, y0: float = 0, size: float = 1.0, stroke: float = 9, gap: float = 16) -> tuple[str, float]:
    strokes: list[str] = []
    fills: list[str] = []
    x = 0.0
    for ch in "beholder":
        s, fl, adv = glyph(ch, x)
        strokes += s
        fills += fl
        x += adv + gap
    width = x - gap
    body = (
        f'<g transform="translate({f(x0)} {f(y0)}) scale({f(size)})">'
        f'<g fill="none" stroke="{color}" stroke-width="{f(stroke)}" stroke-linecap="butt" stroke-linejoin="round">'
        + "".join(f'<path d="{d}"/>' for d in strokes)
        + f'</g><g fill="{color}">{"".join(fills)}</g></g>'
    )
    return body, width * size


# --- documents --------------------------------------------------------------


def svg(w, h, body, title="beholder", extra=""):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(w)} {f(h)}" width="{f(w)}" height="{f(h)}" role="img" aria-label="{title}">'
        f"<title>{title}</title>{extra}{body}</svg>\n"
    )


def adaptive_style(cls: str) -> str:
    return (
        f"<style>.{cls}{{color:{INK_LIGHT}}}"
        f"@media (prefers-color-scheme: dark){{.{cls}{{color:{INK_DARK}}}}}</style>"
    )


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    files: dict[str, str] = {}

    # Mark, square, transparent.
    files["mark-light.svg"] = svg(512, 512, mark_group(INK_LIGHT))
    files["mark-dark.svg"] = svg(512, 512, mark_group(INK_DARK))
    files["mark.svg"] = svg(512, 512, f'<g class="m">{mark_group("currentColor")}</g>', extra=adaptive_style("m"))

    # Horizontal lockup: mark + wordmark.
    def lockup(color):
        m = f'<g transform="translate(0 0) scale(0.5)">{mark_group(color, uid="lockup-pupil")}</g>'
        wm, ww = wordmark(color, x0=304, y0=168, size=1.25, stroke=9)
        return m + wm, 304 + ww + 8

    body, w = lockup(INK_LIGHT)
    files["logo-light.svg"] = svg(w, 256, body)
    body, w = lockup(INK_DARK)
    files["logo-dark.svg"] = svg(w, 256, body)
    body, w = lockup("currentColor")
    files["logo.svg"] = svg(w, 256, f'<g class="m">{body}</g>', extra=adaptive_style("m"))

    # Favicon: reduced detail, adapts to the browser's color scheme.
    files["favicon.svg"] = svg(
        512, 512, f'<g class="m">{mark_group("currentColor", detail="small", uid="fav-pupil")}</g>', extra=adaptive_style("m")
    )
    # App icon: reduced detail on a solid tile (for apple-touch and PWA icons).
    files["app-icon.svg"] = svg(
        512,
        512,
        f'<rect width="512" height="512" fill="{BG_DARK}"/>'
        f'<g transform="translate(56 56) scale(0.78125)">{mark_group(INK_DARK, detail="small", uid="app-pupil")}</g>',
    )

    # Social cards: 1280x640 for GitHub, 1200x630 for wyrd.foo (Open Graph).
    def social(w: int, h: int, mark_x: float, text_x: float, size: float) -> str:
        grid = "".join(
            f'<rect x="{x}" y="{y}" width="4" height="4" rx="1"/>' for x in range(20, w, 40) for y in range(20, h, 40)
        )
        k = 432 / 512
        my = (h - 512 * k) / 2
        cy = h / 2
        wm, _ = wordmark(INK_DARK, x0=text_x, y0=cy - 6, size=size, stroke=10)
        return svg(
            w,
            h,
            f'<rect width="{w}" height="{h}" fill="{BG_DARK}"/>'
            f'<g fill="{INK_DARK}" opacity="0.06">{grid}</g>'
            f'<g transform="translate({f(mark_x)} {f(my)}) scale({f(k)})">{mark_group(INK_DARK, uid="social-pupil")}</g>'
            + wm
            + f'<text x="{f(text_x)}" y="{f(cy + 68)}" fill="{INK_DARK}" opacity="0.72" '
            'font-family="Inter, Segoe UI, Helvetica, Arial, sans-serif" font-size="30">'
            f'<tspan x="{f(text_x)}">See structural risk and</tspan>'
            f'<tspan x="{f(text_x)}" dy="42">dependency relationships in code.</tspan></text>',
        )

    files["social-preview.svg"] = social(1280, 640, 64, 600, 1.5)
    files["social-image.svg"] = social(1200, 630, 48, 552, 1.45)

    for name, text in files.items():
        (OUT / name).write_text(text)
        print(OUT / name)


if __name__ == "__main__":
    main()
