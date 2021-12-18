#[test]
fn test_adjacent() {
    use crate::game::Position;

    let pos_1 = Position::from_xy(10, 10);
    let pos_2 = Position::from_xy(10, 10);
    let pos_3 = Position::from_xy(10, 9);
    let pos_4 = Position::from_xy(10, 11);
    let pos_5 = Position::from_xy(9, 10);
    let pos_6 = Position::from_xy(11, 10);
    let pos_7 = Position::from_xy(11, 11);
    let pos_8 = Position::from_xy(9, 9);
    let pos_9 = Position::from_xy(11, 9);
    let pos_10 = Position::from_xy(9, 11);
    let pos_11 = Position::from_xy(10, 8);
    let pos_12 = Position::from_xy(10, 12);
    let pos_13 = Position::from_xy(8, 10);
    let pos_14 = Position::from_xy(12, 10);

    assert!(!pos_1.is_adjacent(&pos_2));
    assert!(pos_1.is_adjacent(&pos_3));
    assert!(pos_1.is_adjacent(&pos_4));
    assert!(pos_1.is_adjacent(&pos_5));
    assert!(pos_1.is_adjacent(&pos_6));
    assert!(!pos_1.is_adjacent(&pos_7));
    assert!(!pos_1.is_adjacent(&pos_8));
    assert!(!pos_1.is_adjacent(&pos_9));
    assert!(!pos_1.is_adjacent(&pos_10));
    assert!(!pos_1.is_adjacent(&pos_11));
    assert!(!pos_1.is_adjacent(&pos_12));
    assert!(!pos_1.is_adjacent(&pos_13));
    assert!(!pos_1.is_adjacent(&pos_14));
}

#[test]
fn test_move_update() {
    use crate::game::Position;

    let mut start_1 = Position::from_xy(1, 2);
    let end_1 = Position::from_xy(5, 6);
    start_1.move_to_xy(5, 6);
    start_1.update();
    assert_eq!(start_1, end_1);
}
