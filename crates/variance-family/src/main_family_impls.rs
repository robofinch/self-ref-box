use crate::traits::{ContravariantFamily, CovariantFamily, LifetimeFamily};


trait ImplyBound<Foo, Bar> {
    type SelfAlias: ?Sized;
}

impl<T: ?Sized, Foo, Bar> ImplyBound<Foo, Bar> for T {
    type SelfAlias = Self;
}


impl<'a, 'lower, 'upper, T> LifetimeFamily<'lower, 'upper> for &'a T
where
    T: LifetimeFamily<'lower, 'upper>,
    for<'varying> T::WithLifetime<'varying>: 'a,
{
    type WithLifetime<'varying> = &'a T::WithLifetime<'varying>
    where
        'upper: 'varying,
        'varying: 'lower;
}

fn test<'lower, 'upper, T: LifetimeFamily<'lower, 'upper>>(t: T) {
}

fn test_driver() {
    test::<'_, 'static, u8>(0);

    let foo: &'static u8 = &0;
    test::<'static, 'static, &'static u8>(foo);
}
