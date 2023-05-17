use crate::assets;
use crate::tiles::Tile;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TILE: Tile = Tile::new("empty", assets::symbols::movement::EMPTY);
}
