# LinkedIn thumbnail

- `rusty-dlp-linkedin.png`: 2400 × 1260 PNG for uploading.
- `rusty-dlp-linkedin-1200.png`: 1200 × 630 PNG.
- `render_thumbnail.py`: editable Pillow renderer; no server or image-generation service.

Run `python artifacts/social/render_thumbnail.py` from the repository. Pillow is required. The renderer uses the existing app icon and local font files, so rerendering needs no network access.

Geist and Geist Mono are from the [Google Fonts repository](https://github.com/google/fonts), under the accompanying SIL Open Font Licenses. The artwork uses the project's original logo; it does not imply Vercel authorship or endorsement.
