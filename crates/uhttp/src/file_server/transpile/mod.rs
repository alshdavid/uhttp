//! JIT transpilation of sources into something a browser can run.
//!
//! The transpilers are self-contained: [`typescript`] turns TypeScript and
//! TSX into JavaScript and [`scss`] turns SCSS and Sass into CSS. Each exposes
//! a transformer that can be used on its own.

pub mod scss;
pub mod typescript;
