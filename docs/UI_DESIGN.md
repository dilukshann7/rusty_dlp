# rusty_dlp appearance

The desktop interface combines the [YouTube design reference](https://github.com/Meliwat/awesome-ios-design-md/tree/main/design-md/video/youtube) with the principles in the [Apple Design skill](https://github.com/emilkowalski/skills/blob/main/skills/apple-design/SKILL.md).

## Media and hierarchy

- Neutral charcoal and soft white themes, compact metadata, and medium-weight titles keep the focus on media.
- Rounded 16:9 thumbnails carry a dark duration badge. Unknown durations display an em dash.
- Red is reserved for the app's small play mark. Download actions use high-contrast neutral pills; blue identifies focus and progress.
- A floating sidebar provides navigation, queue counts, and activity at a glance. New download keeps its selection count and queue action outside the scrolling content.
- Quality, media type, destination, and playlist folders remain visible. **More options** reveals exact streams, filename templates, presets, and saved defaults. Collapsing these controls preserves their values and indicates when an exact selection remains active.

## Glass material

Shared styles in `src/ui/material.rs` provide translucent gradients, light-catching edges, soft shadows, and restrained, static ambient color. The sidebar uses a denser material than content panels; inputs use solid surfaces for readability. Controls have hover, pressed, disabled, and focus states where supported by iced.

This is a cross-platform iced interpretation of Liquid Glass. It uses alpha composition over a cached background image; it does not implement native Apple vibrancy, live backdrop blur, or optical refraction. No additional OS-specific compositor is required.

**Settings → Reduce transparency** replaces glass panels with opaque surfaces, strengthens surface boundaries, removes their shadows, and hides ambient lighting. The preference persists with the other settings. Dark, light, and system appearance remain available.

The app uses the system's default sans-serif font, 14 px body controls, 13 px metadata, 18 px section titles, and 24 px screen titles. Controls target a 44 px height, and the existing 900 × 600 minimum window size remains in place.

## Verification boundary

Source formatting, manifest consistency, and offline packaging checks can run without compiling or launching the app. Visual checks at the minimum window size, keyboard navigation, and platform rendering still require an explicitly authorized app launch. The material is not claimed to be visually verified on any platform until those checks occur.

For this change, formatting, locked manifest metadata, and all three offline packaging fixture tests passed. Existing editor diagnostics exposed an iced progress-bar API mismatch; the control now uses the documented `girth` setter. No compiler, Rust tests, or GUI were launched by the implementing agent. A separate review worker could not complete because it reached its service usage limit; the final changes received local source inspection only.
