#[test]
fn test_dna_encoding() {
    use crate::entity::dna::{
        ActionPrototype, Actuators, GeneLibrary, Processors, Sensors, TraitAction,
    };
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
        actions:     Vec::new(),
        sense_range: 1,
    };

    let p = Processors {
        actions: vec![ActionPrototype {
            trait_id:  TraitAction::Quick,
            parameter: 1,
        }],
    };

    let a = Actuators {
        actions: vec![ActionPrototype {
            trait_id:  TraitAction::Move,
            parameter: 1,
        }],
        hp:      0,
    };

    let (_s, _p, _a) = gene_lib.decode_dna(&dna);
    println!("{:?}", _s);
    println!("{:?}", _p);
    println!("{:?}", _a);
    assert_eq!(s, _s);
    assert_eq!(p, _p);
    assert_eq!(a, _a);
}
