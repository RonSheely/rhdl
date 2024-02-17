use anyhow::bail;
use anyhow::Result;
use rhdl_bits::alias::*;
use rhdl_bits::{bits, Bits};
use rhdl_core::note;
use rhdl_core::Tristate;
use rhdl_macro::{kernel, Digital};

use crate::circuit::root_descriptor;
use crate::circuit::root_hdl;
use crate::circuit::BufZ;
use crate::{circuit::Circuit, clock::Clock, constant::Constant, dff::DFF};

// Build a strobe
#[derive(Clone)]
pub struct Strobe<const N: usize> {
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(param: Bits<N>) -> Self {
        Self {
            threshold: param.into(),
            counter: DFF::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeQ<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::O,
    counter: <DFF<Bits<N>> as Circuit>::O,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeD<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::I,
    counter: <DFF<Bits<N>> as Circuit>::I,
}

#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub struct StrobeZ<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::Z,
    counter: <DFF<Bits<N>> as Circuit>::Z,
}

impl<const N: usize> Tristate for StrobeZ<N> {
    const N: usize = <Constant<Bits<N>> as Circuit>::Z::N + <DFF<Bits<N>> as Circuit>::Z::N;
}

impl<const N: usize> rhdl_core::Notable for StrobeZ<N> {
    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
        self.threshold.note(key, &mut writer);
        self.counter.note(key, &mut writer);
    }
}

impl<const N: usize>
    From<(
        <Constant<Bits<N>> as Circuit>::O,
        <DFF<Bits<N>> as Circuit>::O,
    )> for StrobeQ<N>
{
    fn from(
        (threshold, counter): (
            <Constant<Bits<N>> as Circuit>::O,
            <DFF<Bits<N>> as Circuit>::O,
        ),
    ) -> Self {
        Self { threshold, counter }
    }
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeI {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> Circuit for Strobe<N> {
    type I = StrobeI;

    type O = bool;

    type Q = StrobeQ<N>;

    type D = StrobeD<N>;

    type S = (
        Self::Q,
        <Constant<Bits<N>> as Circuit>::S,
        <DFF<Bits<N>> as Circuit>::S,
    );

    type Z = StrobeZ<N>;

    type Update = strobe<N>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = strobe::<N>;

    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O {
        loop {
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            state.0.threshold =
                self.threshold
                    .sim(internal_inputs.threshold, &mut state.1, &mut io.threshold);
            state.0.counter =
                self.counter
                    .sim(internal_inputs.counter, &mut state.2, &mut io.counter);
            if state == &prev_state {
                return outputs;
            }
        }
    }

    fn name(&self) -> &'static str {
        "Strobe"
    }

    fn descriptor(&self) -> crate::circuit::CircuitDescriptor {
        let mut ret = root_descriptor(self);
        ret.children
            .insert("threshold".to_string(), self.threshold.descriptor());
        ret.children
            .insert("counter".to_string(), self.counter.descriptor());
        ret
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> Result<crate::circuit::HDLDescriptor> {
        let mut ret = root_hdl(self, kind)?;
        ret.children
            .insert("threshold".to_string(), self.threshold.as_hdl(kind)?);
        ret.children
            .insert("counter".to_string(), self.counter.as_hdl(kind)?);
        Ok(ret)
    }
}

#[kernel]
pub fn strobe<const N: usize>(i: StrobeI, q: StrobeQ<N>) -> (bool, StrobeD<N>) {
    let mut d = StrobeD::<N>::default();
    note("i", i);
    note("q", q);
    d.counter.clock = i.clock;
    let counter_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    d.counter.data = counter_next;
    note("out", strobe);
    note("d", d);
    (strobe, d)
}
