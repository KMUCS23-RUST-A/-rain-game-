mod enums;
mod game;

pub use enums::{GameState, Message, WordColor};
pub use game::game::{play, Game};
