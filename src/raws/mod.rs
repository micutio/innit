pub mod spawn;
pub mod templating;

use bracket_lib::prelude as rltk;
use bracket_lib::terminal::EMBED;
use spawn::Spawn;
use templating::ObjectTemplate;

rltk::embedded_resource!(SPAWN_RAW, "../../raws/spawns.json");
rltk::embedded_resource!(OBJECT_RAW, "../../raws/objects.json");

/// Load the spawn data from the `spawns.json` file.
///
/// # Panics
///
/// Panics if the `spawns.json` file cannot be accessed or found.
#[must_use]
pub fn load_spawns() -> Vec<Spawn> {
    rltk::link_resource!(SPAWN_RAW, "../raws/spawns.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = EMBED
        .lock()
        // .unwrap()
        .get_resource("../raws/spawns.json".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(raw_data).expect("Unable to convert to a valid UTF-8 string.");
    serde_json::from_str(raw_string).expect("Unable to parse JSON")
}

/// Load the object templates from the `objects.json` file.
///
/// # Panics
///
/// Panics if the `objects.json` file cannot be accessed or found.
#[must_use]
pub fn load_object_templates() -> Vec<ObjectTemplate> {
    rltk::link_resource!(OBJECT_RAW, "../raws/objects.json");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = EMBED
        .lock()
        // .unwrap()
        .get_resource("../raws/objects.json".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(raw_data).expect("Unable to convert to a valid UTF-8 string.");
    serde_json::from_str(raw_string).expect("Unable to parse JSON")
}
