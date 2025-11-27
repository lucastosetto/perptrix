//! Core application primitives (engines, orchestrators)

pub mod http;
pub mod runtime;
pub mod bootstrap {
    //! Entry points and high-level initialization hooks.
}

pub use http::*;
pub use runtime::*;
