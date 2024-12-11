use rhdl::prelude::*;

use crate::axi4lite::basic::bridge;
use crate::axi4lite::basic::manager;
use crate::core::option::unpack;
use crate::core::ram;

const RAM_ADDR: usize = 8;

// This is a simple test harness that connects a basic manager and subordinate
// into a test fixture.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    manager: manager::read::U,
    subordinate: bridge::read::U,
    memory: ram::synchronous::U<Bits<32>, RAM_ADDR>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            manager: manager::read::U::default(),
            subordinate: bridge::read::U::default(),
            memory: ram::synchronous::U::new((0..256).map(|n| (bits(n), bits(n << 8 | n)))),
        }
    }
}

#[derive(Debug, Digital)]
pub struct I {
    pub cmd: Option<b32>,
}

#[derive(Debug, Digital)]
pub struct O {
    pub data: Option<Bits<32>>,
    pub full: bool,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = basic_test_kernel;
}

#[kernel]
pub fn basic_test_kernel(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    d.memory.write.addr = Bits::<RAM_ADDR>::default();
    d.memory.write.value = bits(0);
    d.memory.write.enable = false;
    d.manager.axi = q.subordinate.axi;
    d.subordinate.axi = q.manager.axi;
    d.manager.cmd = i.cmd;
    d.subordinate.data = q.memory;
    // The read bridge uses a read strobe, but we will ignore that
    // for this test case, since the RAM does not care how many times
    // we read it.
    let (_, axi_addr) = unpack::<Bits<32>>(q.subordinate.read);
    let read_addr = (axi_addr >> 3).resize();
    let mut o = O {
        data: q.manager.data,
        full: q.manager.full,
    };
    if cr.reset.any() {
        o.data = None;
    }
    d.memory.read_addr = read_addr;
    (o, d)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    fn test_stream() -> impl Iterator<Item = TimedSample<(ClockReset, I)>> {
        (0..5)
            .map(|n| Some(bits(n << 3)))
            .chain(std::iter::repeat(None))
            .map(|x| I { cmd: x })
            .take(100)
            .stream_after_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_transaction_trace() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("axi4lite")
            .join("basic");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["84072a32114264ba3c3316f65d2acfc7124b6ec1a0594bc482615d33cebe8864"];
        let digest = vcd.dump_to_file(&root.join("basic_read_test.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_that_reads_are_correct() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let io = uut.run(input)?;
        let io = io
            .synchronous_sample()
            .flat_map(|x| x.value.2.data)
            .collect::<Vec<_>>();
        let expected = (0..256).map(|n| bits(n << 8 | n)).collect::<Vec<_>>();
        assert_eq!(io, expected[0..io.len()]);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::default();
        let input = test_stream();
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
