pub mod regex;

use num_bigint::BigUint;
use std::error::Error;
pub trait Lang<T> {
    type IntoIter: IntoIterator<Item = T>;
    type Error: Error;
    ///Tests the cardinality of a given language, should the languge be infinite, this function will return None.
    fn cardinality(&self) -> Option<usize> {
        None
    }
    ///Associate a given instance of langauge with an integer.
    fn state<I: IntoIterator<Item = T>>(&self, instance: I) -> Result<BigUint, Self::Error>;
    ///The inverse of the state function.
    fn instance(&self, state: BigUint) -> Result<Self::IntoIter, Self::Error>;
}
