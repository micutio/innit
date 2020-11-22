use crate::core::world::world_gen::is_explored;
use crate::entity::object::Object;
use crate::game::Game;
use rltk::{to_cp437, ColorPair, DrawBatch, Point, Rltk};

pub fn render(game: &mut Game, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    let mut to_draw: Vec<&Object> = game
        .objects
        .get_vector()
        .iter()
        .flatten()
        .filter(|o| {
            // FIXME: there must be a better way than using `and_then`.
            frontend.fov.is_in_fov(o.pos.x, o.pos.y)
                || o.physics.is_always_visible
                || (o.tile.is_some() && *o.tile.as_ref().and_then(is_explored).unwrap())
                || (o.tile.is_some() && game.state.env.debug_mode)
        })
        .collect();

    // sort, so that non-blocking objects come first
    to_draw.sort_by(|o1, o2| o1.physics.is_blocking.cmp(&o2.physics.is_blocking));
    // draw the objects in the list
    for object in &to_draw {
        draw_batch.set(
            Point::new(object.pos.x, object.pos.y),
            ColorPair::new(object.visual.color, rltk::BLACK),
            to_cp437(object.visual.glyph),
        );
    }
}
