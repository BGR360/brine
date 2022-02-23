use smallvec::SmallVec;

use crate::bakery::models::BakedModel;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct HalfBakedBlockState {
    pub models: SmallVec<[HalfBakedBlockStateGrabBag; 1]>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct HalfBakedBlockStateGrabBag {
    pub choices: SmallVec<[HalfBakedGrabBagChoice; 1]>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct HalfBakedGrabBagChoice {
    pub model: BakedModel,
    pub weight: u32,
}
