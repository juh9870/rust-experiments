use crate::tiles::Tile;
use lazy_static::lazy_static;

mod empty;
pub use empty::TILE as EMPTY;

lazy_static! {
    pub static ref ALL: Vec<Tile> = vec![EMPTY.clone()];
}
