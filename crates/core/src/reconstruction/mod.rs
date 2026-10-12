//! Recovery metadata. Catalog entries do not execute reconstructed game logic.

mod catalog;
pub use catalog::ROUTINES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Translation {
    Recovered,
    Partial,
    SourcePort,
}

#[derive(Debug)]
pub struct Routine {
    pub name: &'static str,
    pub entry: u32,
    pub decompiled: bool,
    pub blocks: usize,
    pub instructions: usize,
    pub translation: Translation,
    pub rust: &'static [&'static str],
    pub calls: &'static [u32],
}
