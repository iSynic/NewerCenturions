# MACE3 decoder attribution and changes

The Rust decoder in `crates/content/src/audio/mace3.rs` adapts
`MACEExp1to3` and its coefficient tables from Burgerlib, by Rebecca Ann
Heineman (copyright 1995-2017), at commit
`fb9ce50a22d23066466879d2d08f6014e40e7449`:

https://github.com/Olde-Skuul/burgerlib/blob/fb9ce50a22d23066466879d2d08f6014e40e7449/source/audio/brmace.cpp

Burgerlib's full upstream permission and warranty notice is in `LICENSE`.
The Rust source retains the applicable notice and author credit. This is an
altered source version, not the original Burgerlib implementation.

Local changes:

- Rust implementation limited to mono MACE 3:1, without streaming state.
- Signed 16-bit little-endian PCM output, repeating each decoded signed
  sample byte in both bytes of the sample.
- Preserve exactly -32768 at the negative saturation boundary; sums below
  -32768 still become -32767. The independent whole-PCM fixture demonstrates
  this boundary in the supplied sound 10121.
- Bounded compressed sound-resource parsing and rejection of unsupported
  headers are implemented separately in `crates/content/src/audio.rs`.

The implementation does not include FFmpeg source or tables. FFmpeg is an
external fixture-generation reference; it is not a runtime dependency.
`evidence/sound-mace3.md` records the header contract and verification scope.
