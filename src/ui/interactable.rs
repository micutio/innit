use tcod::input::Event;

/// Interactables are UI elements that can be interacted with via the mouse.
/// To realise that, this trait provides the interface for a callback that can handle mouse events.
trait Interactable {
    /// Returns the element layout (min_x, min_y, max_x, max_y) as absolute screen coordinates.
    fn get_layout_abs(&self) -> (i32, i32, i32, i32);

    /// Returns the element layout (min_x, min_y, max_x, max_y) as coordinates relative to the
    /// parent interactable. If there is no parent interactable this is identical to
    /// `get_layout_abs()`
    fn get_layout_rel(&self) -> (i32, i32, i32, i32);

    /// Handle a mouse event.
    // TODO: How to do different return values, e.g.: selected option or no return values?
    fn callback(&self, event: Event);
}

pub struct BottomPanel {
    ui_elements: Vec<Box<dyn Interactable>>,
}

impl BottomPanel {
    fn new() -> Self {
        BottomPanel {
            ui_elements: Vec::new(),
        }
    }
}

impl Interactable for BottomPanel {
    fn get_layout_abs(&self) -> (i32, i32, i32, i32) {
        unimplemented!()
    }

    fn get_layout_rel(&self) -> (i32, i32, i32, i32) {
        unimplemented!()
    }

    fn callback(&self, event: Event) {
        // unimplemented!()
        if let Event::Mouse(m) = event {
            if m.lbutton_pressed {
                unimplemented!()
            }
        }
    }
}
