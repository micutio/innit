use crate::core::game_objects::GameObjects;
use crate::core::game_state::GameState;
use crate::ui::dialog::InfoBox;

pub fn character_screen(state: &GameState, objects: &GameObjects) -> InfoBox {
    if let Some(ref player) = objects[state.player_idx] {
        let title: String = "Character Information".to_string();
        let lines = vec![
            format!(
                "Energy:      {}/{}",
                player.processors.energy, player.processors.energy_storage
            ),
            format!("Metabolism:  {}", player.processors.metabolism),
            format!("Sense Range: {}", player.sensors.sensing_range),
            format!("HP:          {}", player.actuators.max_hp),
            format!("Alive:       {}", player.alive),
            format!("Turn:        {}", state.turn),
        ];
        InfoBox::new(title, lines)
    } else {
        panic!("No player");
    }
}
