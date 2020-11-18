use tcod::input::Event;

/// Interactables are UI elements that can be interacted with via the mouse.
/// To realise that, this trait provides the interface for a callback that can handle mouse events.
trait Interactable {

    /// Returns the element layout (min_x, min_y, max_x, max_y) as absolute screen coordinates.
    fn get_layout_abs() -> (i32, i32, i32, i32);

    /// Returns the element layout (min_x, min_y, max_x, max_y) as coordinates relative to the
    /// parent interactable. If there is no parent interactable this is identical to
    /// `get_layout_abs()`
    fn get_layout_rel() -> (i32, i32, i32, i32);

    /// Handle a mouse event.
    // TODO: How to do different return values, e.g.: selected option or no return values?
    fn callback(event: Event::Mouse);
}