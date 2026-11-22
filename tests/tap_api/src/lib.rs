//! Test API of the tap decoder's end-to-end tests (TR-14): calls, a long reply, properties, a global trait and a
//! quantity whose unit the device names by reference to a unit of this crate.
#![cfg_attr(not(feature = "std"), no_std)]

use ww_api::prelude::*;
use ww_si::prefix::{Centi, One};
use ww_si::units::Ratio;
use ww_si::{AnyQuantity, Fixed};

#[ww_api_root]
pub trait TapDemo {
    /// `a + b`, wrapping
    #[effect(read)]
    fn add(a: u32, b: u32) -> u32;
    /// `len` bytes counting up from 0: a long reply is split over several USB packets
    #[effect(read)]
    fn blob(len: u16) -> RefVec<'i, u8>;
    #[effect(edit)]
    property!(rw speed: Fixed<Rpm, One, u16>);
    /// Changes on every poll of the device, observed
    property!(ro+observe load: Fixed<Ratio, Centi, u16>);
    /// Observed together with `load`: an update carries both
    property!(ro+observe state: State);
    /// A reading whose unit the device picks at run time, here [Rpm] by reference
    property!(ro sensor: AnyQuantity<'i>);
    /// A global trait (known from a snapshot): the device leaves it out of its introspection data
    ww_impl!(user: global_fixture_api::GlobalDemo);
}

ww_si::unit! {
    /// Revolutions per minute: π/30 rad/s
    pub struct Rpm = "rpm", [rad 1, s -1], Exact(1 / 30, pi 1);
}

/// The device's operating state, a raw code
#[derive_shrink_wrap(
    borrowed,
    owned(feature = "std"),
    derive(Clone, Copy, Debug, PartialEq, Eq)
)]
pub struct State(pub u8);
