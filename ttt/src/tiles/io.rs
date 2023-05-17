use crate::tiles::Tile;
use lazy_static::lazy_static;

mod discard;
pub use discard::TILE as DISCARD;

lazy_static! {
    pub static ref ALL: Vec<Tile> = vec![DISCARD.clone()];
}
