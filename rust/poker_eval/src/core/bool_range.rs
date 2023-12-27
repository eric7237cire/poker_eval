use serde::{Deserialize, Serialize};

use crate::InRangeType;

#[derive(Serialize, Deserialize, Default)]
pub struct BoolRange {
    pub data: InRangeType,
}

impl BoolRange {
    pub fn new(data: InRangeType) -> Self {
        BoolRange { data }
    }

    //Use holecard -- to_range_index
}