# Miniquad 0.4.11 local patch

Source: the Cargo registry distribution of miniquad 0.4.11, used by Macroquad 0.4.16. Upstream MIT/Apache licenses are preserved.

The Windows key-down and key-up handlers now fall back to `MapVirtualKeyW` with `MAPVK_VK_TO_VSC_EX` when a message contains no scan code. Physical scan codes retain the existing path, including extended arrow/numpad distinctions. Two Windows tests cover the controls used by this application and unknown input. This addresses an observed native input failure: virtual-key input arrived as `KeyCode::Unknown` and game controls did not fire.

Run `cargo test --manifest-path vendor/miniquad/Cargo.toml --lib --target-dir target/miniquad-check` from the root. Mapping semantics follow [Microsoft's MapVirtualKeyW documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-mapvirtualkeyw).

No other upstream source behavior was changed. Java/JavaScript platform helpers and registry metadata are omitted because this delivery targets Windows.
