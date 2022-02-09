//! Minecraft block data.
//!
//! TODO: about block ids and block states.

mod block;
mod state;

pub use block::{Block, BlockId, BlockStateId, Blocks};
pub use state::{BlockState, StateValue};
