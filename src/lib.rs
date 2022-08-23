#![feature(
    concat_idents,
    proc_macro_hygiene,
    simd_ffi
)]
#![allow(
    unused_macros,
    unused_must_use,
    clippy::borrow_interior_mutable_const,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::absurd_extreme_comparisons,
    clippy::cmp_null
)]

mod wario;

#[skyline::main(name = "shoulder_bash")]
pub fn main() {
    wario::install();
}