#[test]
fn test_adjacent() {
    use crate::core::position::Position;

    let pos_1 = Position::new(10, 10);
    let pos_2 = Position::new(10, 10);
    let pos_3 = Position::new(10, 9);
    let pos_4 = Position::new(10, 11);
    let pos_5 = Position::new(9, 10);
    let pos_6 = Position::new(11, 10);
    let pos_7 = Position::new(11, 11);
    let pos_8 = Position::new(9, 9);
    let pos_9 = Position::new(11, 9);
    let pos_10 = Position::new(9, 11);
    let pos_11 = Position::new(10, 8);
    let pos_12 = Position::new(10, 12);
    let pos_13 = Position::new(8, 10);
    let pos_14 = Position::new(12, 10);

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
