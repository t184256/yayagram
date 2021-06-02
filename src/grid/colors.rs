use terminal::util::Color;

/// Every second batch of clues is highlighted with this gray color.
/// It is supposed to make reading the clues and knowing which cells they point at easier.
pub const HIGHLIGHTED_CLUE_BACKGROUND_COLOR: Color = Color::Byte(238);

pub const SOLVED_CLUE_COLOR: Color = Color::Byte(244);
