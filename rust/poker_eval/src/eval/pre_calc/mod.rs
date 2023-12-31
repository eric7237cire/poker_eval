mod constants;
pub mod fast_eval;

mod lookup;

//This is just used to generate the lookup tables
#[allow(dead_code)]
#[cfg(not(target_arch = "wasm32"))]
mod lookup_tables;

#[cfg(not(target_arch = "wasm32"))]
mod get_env;
#[cfg(not(target_arch = "wasm32"))]
pub use get_env::*;

pub mod perfect_hash;

pub use constants::*;

mod boom;
pub mod rank;
