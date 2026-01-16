use core::fmt::{Debug, Formatter, Result as FmtResult};

use variance_family::WithLifetime;


pub enum SelfRefSlot<'varying, N, S, E, Upper>
where
    S: WithLifetime<'varying, 'varying, Upper, Is: Sized>,
    E: WithLifetime<'varying, 'varying, Upper, Is: Sized>,
    Upper: ?Sized,
{
    NoRef(N),
    SharedRef(S::Is),
    ExclusiveRef(E::Is),
}

impl<'varying, N, S, E, Upper> Clone for SelfRefSlot<'varying, N, S, E, Upper>
where
    N: Clone,
    S: WithLifetime<'varying, 'varying, Upper, Is: Clone>,
    E: WithLifetime<'varying, 'varying, Upper, Is: Clone>,
    Upper: ?Sized,
{
    fn clone(&self) -> Self {
        match self {
            Self::NoRef(no_ref) => Self::NoRef(no_ref.clone()),
            Self::SharedRef(shared_ref) => Self::SharedRef(shared_ref.clone()),
            Self::ExclusiveRef(exclusive_ref) => Self::ExclusiveRef(exclusive_ref.clone()),
        }
    }
}

impl<'varying, N, S, E, Upper> Copy for SelfRefSlot<'varying, N, S, E, Upper>
where
    N: Copy,
    S: WithLifetime<'varying, 'varying, Upper, Is: Copy>,
    E: WithLifetime<'varying, 'varying, Upper, Is: Copy>,
    Upper: ?Sized,
{}

impl<'varying, N, S, E, Upper> Debug for SelfRefSlot<'varying, N, S, E, Upper>
where
    N: Debug,
    S: WithLifetime<'varying, 'varying, Upper, Is: Sized + Debug>,
    E: WithLifetime<'varying, 'varying, Upper, Is: Sized + Debug>,
    Upper: ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::NoRef(no_ref) => {
                f.debug_tuple("NoRef").field(no_ref).finish()
            }
            Self::SharedRef(shared_ref) => {
                f.debug_tuple("SharedRef").field(shared_ref).finish()
            }
            Self::ExclusiveRef(exclusive_ref) => {
                f.debug_tuple("ExclusiveRef").field(exclusive_ref).finish()
            }
        }
    }
}
