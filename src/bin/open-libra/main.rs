//! Main entry point for OpenLibra

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use open_libra::application::APPLICATION;

/// Boot OpenLibra
fn main() {
    abscissa_core::boot(&APPLICATION);
}
