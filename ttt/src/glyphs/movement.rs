use crate::glyphs::Glyph;
use lazy_static::lazy_static;

mod empty;
pub use empty::GLYPH as EMPTY;

lazy_static! {
    pub static ref ALL: Vec<Glyph> = vec![EMPTY.clone()];
}
