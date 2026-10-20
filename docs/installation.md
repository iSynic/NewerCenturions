# Installation notes

Download the ZIP for your platform from the
[latest release](https://github.com/iSynic/NewerCenturions/releases/latest).
The original game is required and is not included.

## Original game files

Extract the complete ZIP into your existing New Centurions folder. The new
application goes beside the original Mac application, with the original
Data Files, Scenarios and Warriors folders:

```text
TheNewCenturions/
  New Centurions                 # original Mac application
  New Centurions.rsrc            # preserved resource fork, where applicable
  The New Centurions.exe         # Windows; use the app or binary on other platforms
  Data Files/
  Scenarios/
  Warriors/
  Save/
```

Preserve Macintosh resource forks when extracting or copying the original
installation. Supported formats include raw `.rsrc` sidecars, adjacent
AppleDouble `._filename` sidecars and AppleSingle files. macOS can also read
native resource forks. A copy with only the data forks is insufficient.

There is no extra `NewCenturions` wrapper folder or extraction step. The loader
reads the original installation without modifying it. An explicit folder
argument or `CENTURIONS_ASSET_ROOT` can override automatic discovery.

## Windows

Run **The New Centurions.exe**. If Windows reports a missing `VCRUNTIME140.dll`,
install the Microsoft Visual C++ runtime.

Alt+Enter toggles fullscreen. Changing the resolution preference returns to a
window at the chosen size.

## Linux

Run `./The\ New\ Centurions`. The desktop backend requires X11 or XWayland and
the X11, OpenGL and ALSA runtime libraries. Install `zenity` or `kdialog` for
Save/Open dialogs.

If your extractor drops executable permissions:

```sh
chmod +x 'The New Centurions' centurions-tools
```

## macOS

Choose Apple Silicon (`aarch64`) or Intel (`x86_64`) to match your Mac. Put
**The New Centurions.app** beside the original application and launch it from
Finder. It discovers content beside the bundle.

The experimental bundle has an ad hoc signature, not Developer ID signing or
notarization. macOS may require approval in Privacy & Security. Save/Open uses
the macOS file panels through the system scripting host.

## Saves and preferences

Save and Open use named-file dialogs. Fast Save reuses the last successfully
saved or loaded filename. Cancelling a dialog or loading a malformed save
leaves the current session intact.

New saves are separate from original Mac saves. Original warriors, saves and
game data remain read-only. Imported saves require a new destination before
saving. Complete compatibility with populated original Mac saves remains
unverified. Back up reconstructed saves before changing builds.

Default locations for saves, preferences, created warriors and logs:

| Platform | Location |
| --- | --- |
| Windows | `%LOCALAPPDATA%/NewCenturions` |
| Linux | `$XDG_DATA_HOME/NewCenturions`, normally `~/.local/share/NewCenturions` |
| macOS | `~/Library/Application Support/NewCenturions` |

Command-line overrides include `--save-path`, `--preferences-path` and
`--roster-path`. Keep the same content and resource-fork representation when
moving reconstructed saves between installations; saves validate content identity.

The optional `centurions-tools` executable provides inspection and replay tools.
It is not needed to play. For current behavior and remaining gaps, see the
[reconstruction notes](reconstruction.md#current-functionality-checkpoint).
