//! Read-only original-data acceptance adapters; reports do not grant acceptance.
mod import;
mod resources;
pub(crate) use import::run as import;
pub(crate) use resources::run as resources;
