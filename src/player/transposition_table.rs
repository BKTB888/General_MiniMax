use crate::player::search::EvalResult;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TTBound {
    Exact,
    Lower,
    Upper,
}
#[derive(Copy, Clone, Debug)]
pub struct TTEntry {
    pub(crate) depth: u8,
    pub(crate) value: EvalResult,
    pub(crate) bound: TTBound,
}
