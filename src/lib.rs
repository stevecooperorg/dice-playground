//! Dice Playground — Starlark dice engine + Leptos WASM UI.

#[macro_use]
extern crate starlark;

#[cfg(feature = "cli")]
pub mod cli;

pub mod engine;
pub mod ui;

pub use engine as dice_language;
