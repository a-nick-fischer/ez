use core::fmt::Debug;

use crate::error::TErr;

pub trait EnvModifier<E>: Debug {
    fn apply(&self, env: &mut E) -> Result<(), TErr>;
}
