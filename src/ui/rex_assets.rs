use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON, "../../resources/art/main.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../../resources/art/main.xp");

        RexAssets {
            menu: XpFile::from_resource("../../resources/art/main.xp").unwrap(),
        }
    }
}
