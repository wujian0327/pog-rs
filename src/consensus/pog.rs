use crate::blockchain::path::AggregatedSignedPaths;
use crate::consensus::{Validator, ValidatorError};

pub struct Pog;

impl Pog {
    pub(crate) fn select(
        validators: Vec<Validator>,
        _combines_seeds: [u8; 32],
        _paths: Vec<AggregatedSignedPaths>,
    ) -> Result<Validator, ValidatorError> {
        //TODO
        Ok(validators[0].clone())
    }
}
