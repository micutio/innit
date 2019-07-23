/// Module DNA
///
/// The DNA contains all core information, excluding temporary info such as position etc. This
/// module allows to generate objects from DNA and modify them using mutation as well as crossing.
/// Decoding DNA delivers attributes and functions that fall into one of three categories:
/// perception, orientation (a.k.a. processing), actuation.
///
/// A DNA Genome is implemented as a string of hexadecimal numbers. The start of a gene is marked
/// by the number zero. Genes can overlap, so that parsing the new gene resumes "in the middle" of
/// a previous gene. The genes should be small and encoding the presence of a quality. Versatility
/// is then controlled by the cumulative occurrence of a gene.
/// Basically: the more often a gene occurs, the stronger it's trait will be.
///
/// List of potential genes:
///
/// | Gene | Primary Trait | Trait Attributes | Potential Syngery    | Potential Anti-Synergy |
/// | ---- | ------------- | ---------------- | -------------------- | ---------------------- |
/// |      | sensing       | range, accuracy  | movement (organelle) | camouflage?            |
/// |      | movement      | speed, direction | sensing (organelle)  | camouflage?            |
/// |      | attack        | energy, damage   |                      | defense                |
/// |      | defense       | membrane         |                      | attack                 |
/// |      | camouflage    | energy, accuracy |                      |                        |
///
// TODO: Design a DNA parser and a mapping from symbol to trait struct.
// TODO: Can behavior be encoded in the genome too i.e., fight or flight?
struct DNA {
    sequence: String,
}
