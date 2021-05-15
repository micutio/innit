use crate::ui::dialog::InfoBox;

pub fn controls_screen() -> InfoBox {
    let title: String = "Controls".to_string();
    let lines = vec![
        "Actions".to_string(),
        "↑, ↓, →, ←, left mouse   primary action".to_string(),
        "W, A, S, D               secondary action".to_string(),
        "Q                        first quick action".to_string(),
        "E                        second quick action".to_string(),
        "".to_string(),
        "Reassign Actions".to_string(),
        "CTRL + P                 set primary".to_string(),
        "CTRL + S                 set secondary".to_string(),
        "CTRL + Q                 set first quick".to_string(),
        "CTRL + E                 set second quick".to_string(),
        "".to_string(),
        "Other".to_string(),
        "C                        display character info".to_string(),
        "F1                       display controls".to_string(),
    ];
    InfoBox::new(title, lines)
}
