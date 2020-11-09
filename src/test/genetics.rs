use crate::entity::genetics::DnaType;

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
        gray_code[4], // sensing range
        0x00,
        0x01,
        gray_code[5], // enzyme
        0x00,
        0x01,
        gray_code[1], // move action
    ];
    // create artifical sensor component for comparison
    let s = Sensors {
        actions: Vec::new(),
        sensing_range: 2,
    };

    let p = Processors {
        actions: Vec::new(),
        metabolism: 1,
        energy_storage: 1,
        energy: 0,
        receptors: Vec::new(),
    };

    let a = Actuators {
        actions: vec![Box::new(MoveAction::new())],
        max_hp: 1,
        hp: 1,
    };

    let (_s, _p, _a, _) = gene_lib.decode_dna(DnaType::Nucleus, &dna);

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
    println!("{:#?}", _p.actions);
    assert_eq!(p.actions.len(), _p.actions.len());
    assert_eq!(a.max_hp, _a.max_hp);
    // let a_match_errors = a
    //     .actions
    //     .iter()
    //     .zip(_a.actions.iter())
    //     .filter(|(&a, &b)| a.get_target_category() != b.get_target_category())
    //     .count();
    // assert_eq!(a_match_errors, 0);
    assert_eq!(a.actions.len(), _a.actions.len());
}
