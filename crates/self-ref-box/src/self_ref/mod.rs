mod uninhabited;


use variance_family::{CovariantFamily, Varying};

pub use self::uninhabited::{NeverExclusiveRef, NeverNoRef, NeverSharedRef};


pub trait SelfRef<'varying, Upper>
where
    Upper: ?Sized,
    Self: Sized + for<'lower> CovariantFamily<'lower, Upper>,
    for<'lower> Varying<'varying, 'lower, Upper, Self>: Sized,
{}

impl<'varying, Upper, T> SelfRef<'varying, Upper> for T
where
    Upper: ?Sized,
    T: for<'lower> CovariantFamily<'lower, Upper>,
    for<'lower> Varying<'varying, 'lower, Upper, Self>: Sized,
{}
