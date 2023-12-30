mod constants;
mod fast_eval;
mod get_env;
mod lookup;

//This is just used to generate the lookup tables
#[cfg(not(target_arch = "wasm32"))]
mod lookup_tables;

#[cfg(not(target_arch = "wasm32"))]
pub use get_env::*;

pub mod perfect_hash;

pub use constants::*;

mod rank;
mod boom;