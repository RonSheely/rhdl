use rhdl::prelude::*;

use crate::core::{constant, dff, option::unpack, slice::lsbs};

/// A burst FIFO drainer.  Uses the same sequence as a matching FIFO filler to check that the
/// values returned from the FIFO are the same ones generated by the writer.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<N: BitWidth> {
    rng: crate::rng::xorshift::U,
    sleep_counter: dff::U<Bits<W4>>,
    sleep_len: constant::U<Bits<W4>>,
    read_probability: constant::U<Bits<W16>>,
    valid: dff::U<bool>,
}

impl<N: BitWidth> Default for U<N> {
    fn default() -> Self {
        Self {
            rng: crate::rng::xorshift::U::default(),
            sleep_counter: dff::U::new(bits(0)),
            sleep_len: constant::U::new(bits(4)),
            read_probability: constant::U::new(bits(0xD000)),
            valid: dff::U::new(true),
        }
    }
}

impl<N: BitWidth> U<N> {
    pub fn new(sleep_len: u8, read_probability: u16) -> Self {
        Self {
            rng: crate::rng::xorshift::U::default(),
            sleep_counter: dff::U::new(bits(0)),
            sleep_len: constant::U::new(bits(sleep_len as u128)),
            read_probability: constant::U::new(bits(read_probability as u128)),
            valid: dff::U::new(true),
        }
    }
}

#[derive(Debug, Digital)]
pub struct I<N: BitWidth> {
    pub data: Option<Bits<N>>,
}

#[derive(Debug, Digital)]
pub struct O {
    pub next: bool,
    pub valid: bool,
}

impl<N: BitWidth> SynchronousIO for U<N> {
    type I = I<N>;
    type O = O;
    type Kernel = drain_kernel<N>;
}

#[kernel]
pub fn drain_kernel<N: BitWidth>(cr: ClockReset, input: I<N>, q: Q<N>) -> (O, D<N>) {
    let mut d = D::<N>::dont_care();
    let mut o = O::dont_care();
    // Compute an is-valid bit that is latching
    let was_valid = q.valid || cr.reset.any();
    // By default, the valid bit is latching
    d.valid = was_valid;
    // If there is data available and we are not sleeping, then read the next
    // value.  Validate against the RNG, and advance the rNG
    let (data_available, data) = unpack::<Bits<N>>(input.data);
    let validation = lsbs::<{ N }, 32>(q.rng);
    let data_matches = data == validation;
    let will_read = data_available && q.sleep_counter == 0;
    trace("data", &data);
    trace("validation", &validation);
    trace("data_matches", &data_matches);
    trace("will_read", &will_read);
    o.valid = was_valid;
    o.next = false;
    d.rng = false;
    d.sleep_counter = q.sleep_counter;
    if will_read {
        d.rng = true;
        o.next = true;
        d.valid = data_matches && was_valid;
        let p = lsbs::<16, 32>(q.rng);
        d.sleep_counter = if p > q.read_probability {
            q.sleep_len
        } else {
            bits(0)
        }
    }
    if q.sleep_counter != 0 {
        d.sleep_counter = q.sleep_counter - 1;
    }
    if cr.reset.any() {
        d.valid = true;
        o.next = false;
        o.valid = true;
    }
    (o, d)
}

#[cfg(test)]
mod tests {

    use rhdl::core::sim::ResetOrData;

    use super::*;

    #[test]
    fn test_drainer_validation_works() {
        let uut = U::<16>::default();
        let mut need_reset = true;
        let mut xorshift = crate::rng::xorshift::XorShift128::default();
        let mut rng_out = xorshift.next().unwrap();
        let mut counter = 0;
        let valid = uut
            .run_fn(
                |output| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
                    }
                    if output.next {
                        rng_out = xorshift.next().unwrap();
                        counter += 1;
                        if counter == 4 {
                            rng_out = 0;
                        }
                    }
                    let next_input = I {
                        data: Some(b16((rng_out & 0xFFFF) as u128)),
                    };
                    Some(ResetOrData::Data(next_input))
                },
                100,
            )
            .take(100)
            .synchronous_sample()
            .map(|x| x.value.2.valid)
            .last()
            .unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_drainer() {
        let uut = U::<16>::default();
        let mut need_reset = true;
        let mut xorshift = crate::rng::xorshift::XorShift128::default();
        let mut rng_out = xorshift.next().unwrap();
        let valid = uut
            .run_fn(
                |output| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
                    }
                    if output.next {
                        rng_out = xorshift.next().unwrap();
                    }
                    let next_input = Some(b16((rng_out & 0xFFFF) as u128));
                    Some(ResetOrData::Data(I { data: next_input }))
                },
                100,
            )
            .take(100)
            .synchronous_sample()
            .map(|x| x.value.2.valid)
            .last()
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_drainer_hdl() -> miette::Result<()> {
        let uut = U::<16>::default();
        let mut need_reset = true;
        let mut xorshift = crate::rng::xorshift::XorShift128::default();
        let mut rng_out = xorshift.next().unwrap();
        let test_bench = uut
            .run_fn(
                |output| {
                    if need_reset {
                        need_reset = false;
                        return Some(ResetOrData::Reset);
                    }
                    if output.next {
                        rng_out = xorshift.next().unwrap();
                    }
                    let next_input = Some(b16((rng_out & 0xFFFF) as u128));
                    Some(ResetOrData::Data(I { data: next_input }))
                },
                100,
            )
            .take(100)
            .collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
