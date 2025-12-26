use crate::traits::{ContravariantFamily, CovariantFamily, LifetimeFamily};


impl<'lower, 'upper: 'lower> LifetimeFamily<'lower, 'upper> for u8 {
    type WithLifetime<'varying> = Self where 'upper: 'varying, 'varying: 'lower;
}
