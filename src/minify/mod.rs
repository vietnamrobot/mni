//! Minification implementations for different formats

pub mod css;
pub mod html;
pub mod js;
pub mod json;
pub mod svg;

mod compress;
mod mangle;
mod whitespace;
