use crate::assets;
use crate::tiles::Tile;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TILE: Tile = Tile::new("discard", assets::symbols::io::DISCARD);
}
