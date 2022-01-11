// use crate::rltk::EMBED;
use bracket_lib::prelude as rltk;
use bracket_lib::terminal::EMBED;

rltk::embedded_resource!(SMALL_DUNGEON, "../../resources/art/menu_bg.xp");

pub struct RexAssets {
    pub menu: rltk::XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../../resources/art/menu_bg.xp");

        RexAssets {
            menu: rltk::XpFile::from_resource("../../resources/art/menu_bg.xp").unwrap(),
        }
    }
}
