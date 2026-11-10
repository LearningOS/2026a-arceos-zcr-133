//! Collection types.
//!
//! Re-exports everything in [`alloc::collections`] and adds the hash-based
//! containers, which `alloc` does not provide.

pub use alloc::collections::*;
pub use hashbrown::{HashMap, HashSet};
