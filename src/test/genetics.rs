#[test]
fn test_dna_encoding() {
    use crate::entity::action::MoveAction;
    use crate::entity::genetics::{Actuators, GeneLibrary, Processors, Sensors};
    use crate::util::generate_gray_code;

    // let rng = GameRng::from_seed(RNG_SEED);
    let gray_code = generate_gray_code(4);
    let gene_lib = GeneLibrary::new();
    // encode a single example trait of each super trait
    let dna = vec![
        0x00,
        0x01,
        gray_code[1], // sensing range
        0x00,
        0x01,
        gray_code[3], // quick action
        0x00,
        0x01,
        gray_code[6], // move action
    ];
    // create artifical sensor component for comparison
    let s = Sensors {
        actions: Vec::new(),
        sensing_range: 0,
    };

    let p = Processors { actions: vec![] };

    let a = Actuators {
        actions: vec![Box::new(MoveAction::new())],
        hp: 0,
    };

    let (_s, _p, _a, _) = gene_lib.decode_dna(&dna);
    println!("{:?}", _s);
    println!("{:?}", _p);
    println!("{:?}", _a);

    assert_eq!(s.sensing_range, _s.sensing_range);
    assert_eq!(s.actions.len(), _s.actions.len());
    // TODO: Find a better way of comparing action vectors for equality!
    // let s_match_errors = s
    //     .actions
    //     .iter()
    //     .zip(_s.actions.iter())
    //     .filter(|(&a, &b)| a.get_target_category() != b.get_target_category())
    //     .count();
    // assert_eq!(s_match_errors, 0);

    // let p_match_errors = p
    //     .actions
    //     .iter()
    //     .zip(_p.actions.iter())
    //     .filter(|(&a, &b)| a.get_target_category() != b.get_target_category())
    //     .count();
    // assert_eq!(p_match_errors, 0);
    assert_eq!(p.actions.len(), _p.actions.len());
    assert_eq!(a.hp, _a.hp);
    // let a_match_errors = a
    //     .actions
    //     .iter()
    //     .zip(_a.actions.iter())
    //     .filter(|(&a, &b)| a.get_target_category() != b.get_target_category())
    //     .count();
    // assert_eq!(a_match_errors, 0);
    assert_eq!(a.actions.len(), _a.actions.len());
}
