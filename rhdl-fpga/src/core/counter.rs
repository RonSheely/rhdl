use rhdl::prelude::*;

use super::dff;

// A simple counter that counts the number of boolean true
// values it has seen.  It is parameterized by the number of
// bits in the counter.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {
            count: dff::U::new(Bits::<N>::default()),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = bool;
    type O = Bits<N>;
    type Kernel = counter<N>;
}

#[kernel]
pub fn counter<const N: usize>(cr: ClockReset, enable: bool, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if enable { q.count + 1 } else { q.count };
    let next_count = if cr.reset.any() { bits(0) } else { next_count };
    (q.count, D::<{ N }> { count: next_count })
}

#[cfg(test)]
mod tests {
    use rand::random;

    use super::*;
    use std::{
        iter::{once, repeat},
        path::PathBuf,
    };

    #[test]
    fn test_counter_on_vec() {
        let inputs = (0..100).map(|_| random::<bool>()).collect::<Vec<_>>();
        let inputs = inputs.stream_after_reset(4);
        let inputs = inputs.clock_pos_edge(100);
        let inputs = inputs.collect::<Vec<_>>();
        let uut: U<16> = U::default();
        let output = uut.run(inputs).count();
        assert_eq!(output, 311);
    }

    #[test]
    fn test_counter() -> std::io::Result<()> {
        let inputs_1 = repeat(true).take(100).stream_after_reset(4);
        let inputs_2 = inputs_1.clone();
        let input = inputs_1.chain(inputs_2);
        let input = input.clock_pos_edge(100);
        let uut: U<16> = U::default();
        let vcd: Vcd = uut.run(input).collect();
        vcd.dump_to_file(&PathBuf::from("counter.vcd"))
    }

    #[test]
    fn test_counter_counts_correctly() -> miette::Result<()> {
        // To account for the delay, we need to end with a zero input
        let rand_set = (0..100)
            .map(|_| random::<bool>())
            .chain(once(false))
            .collect::<Vec<_>>();
        let ground_truth = rand_set
            .iter()
            .fold(0, |acc, x| acc + if *x { 1 } else { 0 });
        let stream = rand_set.stream_after_reset(4).clock_pos_edge(100);
        let uut: U<16> = U::default();
        let out_stream = uut.run(stream);
        let output = out_stream.clone().last().map(|x| x.value.2);
        assert_eq!(output, Some(bits(ground_truth)));
        let tb = out_stream.collect::<SynchronousTestBench<_, _>>();
        let tm = tb.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = tb.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
