use crate::glyphs::Glyph;
use lazy_static::lazy_static;

mod discard;
pub use discard::GLYPH as DISCARD;

lazy_static! {
    pub static ref ALL: Vec<Glyph> = vec![DISCARD.clone()];
}
