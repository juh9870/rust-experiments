use crate::assets;
use crate::glyphs::Glyph;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GLYPH: Glyph = Glyph::new("empty", assets::symbols::movement::EMPTY);
}
