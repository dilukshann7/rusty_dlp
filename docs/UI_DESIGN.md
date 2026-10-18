# rusty_dlp appearance

The desktop interface combines the [YouTube design reference](https://github.com/Meliwat/awesome-ios-design-md/tree/main/design-md/video/youtube) with the principles in the [Apple Design skill](https://github.com/emilkowalski/skills/blob/main/skills/apple-design/SKILL.md).

The later spacing and control refinement uses the supplied Vercel guidelines and [Geist color roles](https://vercel.com/geist/colors). These inform the palette and alignment; the app retains its own name and branding.

Dark mode uses a neutral `#0A0A0A` canvas, near-black `#111111` panels, `#1A1A1A` controls, `#333333` borders, `#EDEDED` main text, and `#A1A1A1` secondary text. Ambient lighting is grayscale. Dropdown menus use the same surfaces and borders, with a neutral highlighted row in both themes.

Buttons center their contents vertically. Inputs and dropdown triggers share a 44 px height and 20 px text line height. The two inspection scopes form an equal-width segmented control, centered on both axes. Field gaps are 8 px, control groups use 12–16 px, and panels and the outer shell use 24 px spacing. Headers, scrolling panels, and the persistent action bar share the same left and right edges.

## Media and hierarchy

- Neutral charcoal and soft white themes, compact metadata, and medium-weight titles keep the focus on media.
- Rounded 16:9 thumbnails carry a dark duration badge. Unknown durations display an em dash.
- Red is reserved for the app's small play mark. Download actions use high-contrast neutral buttons; blue identifies focus and progress.
- A floating sidebar provides navigation, queue counts, and activity at a glance. New download keeps its selection count and queue action outside the scrolling content.
- Quality, media type, destination, and playlist folders remain visible. **More options** reveals exact streams, filename templates, presets, and saved defaults. Collapsing these controls preserves their values and indicates when an exact selection remains active.

## Glass material

Shared styles in `src/ui/material.rs` provide restrained translucent gradients, subtle edges, soft panel shadows, and static grayscale ambient lighting. The sidebar uses a denser material than content panels; inputs and dropdown menus use solid surfaces for readability. Controls share 10 px corners and have hover, pressed, disabled, and focus states where supported by iced. Ordinary buttons have no ornamental shadows.

This is a cross-platform iced interpretation of Liquid Glass. It uses alpha composition over a cached background image; it does not implement native Apple vibrancy, live backdrop blur, or optical refraction. No additional OS-specific compositor is required.

**Settings → Reduce transparency** replaces glass panels with opaque surfaces, strengthens surface boundaries, removes their shadows, and hides ambient lighting. The preference persists with the other settings. Dark, light, and system appearance remain available.

The app uses the system's default sans-serif font, 14 px body controls, 13 px metadata, 18 px section titles, and 24 px screen titles. Controls target a 44 px height, and the existing 900 × 600 minimum window size remains in place.

## Verification boundary

Source formatting, manifest consistency, and offline packaging checks can run without compiling or launching the app. Visual checks at the minimum window size, keyboard navigation, and platform rendering still require an explicitly authorized app launch. The material is not claimed to be visually verified on any platform until those checks occur.

The palette and alignment refinement passed `cargo fmt --all -- --check`, `git diff --check`, and a Windows debug build using `cargo build --locked --bin rusty-dlp`, with no compiler errors or warnings. The executable was refreshed after the running app was closed. No app launch or visual verification was performed by the implementing agent, and Rust tests were not run for these appearance changes. Earlier manifest and three offline packaging fixture checks remain separate from GUI verification.
