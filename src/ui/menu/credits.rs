use crate::game::{self, ObjectStore, State};
use crate::ui::dialog;
use crate::ui::menu::{self, Item, Menu};

use bracket_lib::prelude as rltk;

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    ReturnToMain,
}

impl Item for MenuItem {
    fn process(
        _state: &mut State,
        _objects: &mut ObjectStore,
        _menu: &mut Menu<Self>,
        item: &Self,
    ) -> game::RunState {
        match item {
            Self::ReturnToMain => game::RunState::MainMenu(menu::main::new()),
        }
    }
}

pub fn new() -> Menu<MenuItem> {
    Menu::with_header(
        "YOU WON!",
        &[(MenuItem::ReturnToMain, "Return to Main Menu".to_string())],
    )
}

pub fn render_content(ctx: &mut rltk::BTerm) {
    let title = "Credits".to_string();
    let lines = vec![
        " ".to_string(),
        "By Michael Wagner 2018 - 2022".to_string(),
        "Source code available at:".to_string(),
        "https://github.com/Micutio/innit".to_string(),
        " ".to_string(),
    ];

    ctx.set_active_console(game::consts::HUD_CON);
    let layout = rltk::Rect::with_size(0, 0, game::consts::WORLD_WIDTH, game::consts::WORLD_HEIGHT);

    let mut draw_batch = rltk::DrawBatch::new();
    draw_batch.fill_region(
        layout,
        rltk::ColorPair::new((0, 0, 0, 255), (0, 0, 0, 255)),
        rltk::to_cp437(' '),
    );
    draw_batch.submit(game::consts::HUD_CON_Z).unwrap();

    dialog::InfoBox::new(title, lines).render();
}
