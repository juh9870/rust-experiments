pub use all::*;
use asset_macro::generate_assets;

generate_assets! {
    use "assets";
    mod all = ".";
}
