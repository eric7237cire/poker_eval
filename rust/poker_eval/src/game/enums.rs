use crate::BoolRange;

pub type ChipType = u16;

pub enum PreFrabRanges {
    RangeAll,
    Range75,
    Range50,
    Range25,
}

pub fn build_range(range: PreFrabRanges) -> BoolRange {
    match range {
        PreFrabRanges::RangeAll => "22+,A2+,K2+,Q2+,J2+,T2+,92+,82+,72+,62+,52+,42+,32".parse().unwrap(),
        PreFrabRanges::Range75 => "22+, A2s+, K2s+, Q2s+, J2s+, T2s+, 92s+, 82s+, 72s+, 62s+, 52s+, 42s+, A2o+, K2o+, Q2o+, J4o+, T6o+, 96o+, 86o+, 76o".parse().unwrap(),
        PreFrabRanges::Range50 => "22+, A2s+, K2s+, Q2s+, J2s+, T5s+, 96s+, 86s+, 75s+, A2o+, K5o+, Q7o+, J8o+, T8o+".parse().unwrap(),
        PreFrabRanges::Range25 => "55+, A2s+, K5s+, Q8s+, J8s+, T9s, A8o+, K9o+, QTo+, JTo".parse().unwrap(),
    }
}
