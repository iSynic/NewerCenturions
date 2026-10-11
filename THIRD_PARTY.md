# Third-party code and original content

The reconstruction source is licensed under the root MIT license. That license
does not grant rights to the original New Centurions game, artwork, fonts,
sounds, scenario data, manuals, warriors or executable.

Original game content is supplied separately by the player and is not included
in the public repository or runtime packages. Native comparison fixtures and
screenshots can contain original content and remain private local test data.

Miniquad is vendored under its MIT and Apache-2.0 licenses. Its Windows keyboard
changes are documented in `vendor/miniquad/LOCAL_CHANGES.md`.

The MACE3 decoder in `crates/content/src/audio/mace3.rs` is adapted from Rebecca
Ann Heineman's decoder. Its license and attribution are retained in
`vendor/mace3/LICENSE` and `vendor/mace3/LOCAL_CHANGES.md`.

Other Rust dependencies retain their own licenses, recorded by their packages
and the locked dependency versions in `Cargo.lock`.
