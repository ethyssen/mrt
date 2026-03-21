use super::Mechanism;
use super::mechanisms;

/// Returns mechanisms in order. Later mechanisms may assume earlier ones have run.
pub fn all_mechanisms() -> Vec<Box<dyn Mechanism>> {
    vec![
        Box::new(mechanisms::AsRefPath),
        Box::new(mechanisms::SplitImpl),
    ]
}
