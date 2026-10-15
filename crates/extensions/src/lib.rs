//! The extensions built as their own crates (`crates/ext-*`). [`install_all`] adds the ones this
//! build has to the engine, before any `Session` is made.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

/// Installs every extension crate this build has.
pub fn install_all() {
    #[cfg(feature = "mailings")]
    wordcraft_ext_mailings::install();
}

#[cfg(test)]
mod tests;
