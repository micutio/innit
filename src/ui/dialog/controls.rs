use crate::ui::dialog::InfoBox;

pub fn info_screen() -> InfoBox {
    let title: String = "Controls".to_string();
    let lines = vec![
        "Actions".to_string(),
        "↑, ↓, →, ←, left mouse   primary action".to_string(),
        "W, A, S, D               secondary action".to_string(),
        "Q                        first quick action".to_string(),
        "E                        second quick action".to_string(),
        "SPACE                    pass turn".to_string(),
        "".to_string(),
        "Reassign Actions".to_string(),
        "SHIFT + P                set primary".to_string(),
        "SHIFT + S                set secondary".to_string(),
        "SHIFT + Q                set first quick".to_string(),
        "SHIFT + E                set second quick".to_string(),
        "".to_string(),
        "Other".to_string(),
        "C                        display character info".to_string(),
        "F1                       display controls".to_string(),
        "1 - 8                    switch to font <no>".to_string(),
    ];
    InfoBox::new(title, lines)
}
