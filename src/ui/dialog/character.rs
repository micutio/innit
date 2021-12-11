use crate::game::objects::ObjectStore;
use crate::game::State;
use crate::ui::dialog::InfoBox;

pub fn character_screen(state: &State, objects: &ObjectStore) -> InfoBox {
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
        let title: String = "Information".to_string();
        let lines = vec![format!("Turn:        {}", state.turn)];
        InfoBox::new(title, lines)
    }
}
