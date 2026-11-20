# SunnyF3

A Sunrise client mod by Zekrenlight (MIT).
Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence.

A cleaner F3 screen made for Sunrise. While F3 is open, the debug text comes in titled
sections instead of one long list; close F3 and the game is back to normal. Every F3 key
combination, Sunrise's charts (F3+1 to F3+4) and the F3+F6 entry screen keep working.
It shows only while you play with nothing open: the pause menu, inventories, chat and every
other screen hide it. With Frontier installed, its minimap steps aside while F3 is showing.

## Sections
- **Performance**: frame rate (green, yellow or red against your monitor), frame time and the
  1% low, the frame limit, VSync and refresh rate, frame generation, the server's time a tick
  against its budget with TPS (or a server's brand), packets in and out.
- **Position**: X, Y and Z (axis-coloured), block, chunk and the block inside it, facing with
  its axis, yaw and pitch, speed in blocks a second, dimension. Coordinates hide when the
  server asks for reduced debug info.
- **World**: biome by name, light at your feet (red where mobs can spawn), the day and the
  in-game time.
- **Looking At**: the block under the crosshair (name, id, position, properties with true in
  green and false in red) and the creature you look at with its distance.
- **System**: CPU, GPU, graphics API and driver, GPU load, memory, window size.
- **Sunrise**: Sunrise and Minecraft versions, mods running, sections and entities drawn,
  particles, shader pack and Sunrise GPU time, dynamic resolution, HDR, post effects.
- **Debug Entries** (hidden at first): Sunrise's own F3 lines as chosen on F3+F6, mods'
  entries included.

Java and JVM details are left out: Sunrise has no JVM.

## Layout and look
- Each section: Left, Right or Hidden, and its place in the column (1 to 9).
- Style: Panels (titled panels with an accent edge), Strips (a strip per line) or Bare.
- Background opacity, section titles, text shadow, colour values, accent colour, and how often
  the text refreshes (50 to 1000 ms).

## Keys
- Open SunnyF3 Settings: not bound at first (Options > Controls > SunnyF3).
- Settings are also at Options > Mods... > SunnyF3 > Settings.

## Look and settings

SunnyF3 uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).
SunnyF3's Panels style draws its accent edge in the same colour.

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

SunnyF3 works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/sunnyf3.smod
```

`python3 tools/make_icon.py icon.png` redraws the icon.

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
