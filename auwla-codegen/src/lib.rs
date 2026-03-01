mod emitter;

pub use emitter::emit_js;

pub mod expr;
pub mod match_logic; // renamed to match_logic because match is a keyword
pub mod pattern;
pub mod stmt;
pub mod try_logic; // renamed to try_logic for consistency
