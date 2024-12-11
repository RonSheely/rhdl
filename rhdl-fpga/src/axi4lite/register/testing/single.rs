// Create a fixture with a write manager and a read manager and a AXI register
use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<const DATA: usize = 32, const ADDR: usize = 32> {
    writer: crate::axi4lite::basic::manager::write::U<DATA, ADDR>,
    reader: crate::axi4lite::basic::manager::read::U<DATA, ADDR>,
    register: crate::axi4lite::register::single::U<DATA, DATA, ADDR>,
}

#[derive(Digital)]
pub struct I<const DATA: usize, const ADDR: usize> {
    pub write: Option<(Bits<ADDR>, Bits<DATA>)>,
    pub read: Option<Bits<ADDR>>,
}

#[derive(Digital)]
pub struct O<const DATA: usize> {
    pub data: Option<Bits<DATA>>,
}

impl<const DATA: usize, const ADDR: usize> SynchronousIO for U<DATA, ADDR> {
    type I = I<DATA, ADDR>;
    type O = O<DATA>;
    type Kernel = test_kernel<DATA, ADDR>;
}

#[kernel]
pub fn test_kernel<const DATA: usize, const ADDR: usize>(
    _cr: ClockReset,
    i: I<DATA, ADDR>,
    q: Q<DATA, ADDR>,
) -> (O<DATA>, D<DATA, ADDR>) {
    let mut d = D::<DATA, ADDR>::dont_care();
    let mut o = O::<DATA>::dont_care();
    d.writer.cmd = i.write;
    d.reader.cmd = i.read;

    d.register.axi.read = q.reader.axi;
    d.register.axi.write = q.writer.axi;
    d.reader.axi = q.register.axi.read;
    d.writer.axi = q.register.axi.write;

    o.data = q.reader.data;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_cmd<const DATA: usize, const ADDR: usize>(val: i32) -> I<DATA, ADDR> {
        I {
            write: Some((bits(0), bits(val as u128))),
            read: None,
        }
    }

    fn read_cmd<const DATA: usize, const ADDR: usize>() -> I<DATA, ADDR> {
        I {
            write: None,
            read: Some(bits(0)),
        }
    }

    fn no_cmd<const DATA: usize, const ADDR: usize>() -> I<DATA, ADDR> {
        I {
            write: None,
            read: None,
        }
    }

    fn test_stream<const DATA: usize, const ADDR: usize>() -> impl Iterator<Item = I<DATA, ADDR>> {
        [
            write_cmd(1),
            read_cmd(),
            write_cmd(1),
            read_cmd(),
            write_cmd(1),
            write_cmd(1),
            read_cmd(),
        ]
        .into_iter()
        .chain(std::iter::repeat(no_cmd()))
        .take(20)
    }

    #[test]
    fn test_register_trace() -> miette::Result<()> {
        let uut = U::<32, 32>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let vcd = uut.run(input)?.collect::<Vcd>();
        vcd.dump_to_file(&std::path::PathBuf::from("register.vcd"))
            .unwrap();
        Ok(())
    }

    /*


    3425 -> 3270
    3270 -> 3134
    3134 -> 3178
    3178 -> 3209
    3209 -> 3211
    3211 -> 3210
    3210 -> 3216
    3216 -> 3227
    3227 -> 3238
    3238 -> 3233
    3233 -> 3243
    3243 -> 3139
    3139 -> 3260
    3260 -> 2692
    2692 -> 2255
    2255 -> 2322
    2322 -> 2356

    */

    #[test]
    fn generate_dot() -> miette::Result<()> {
        let uut = U::<1, 1>::default();
        let fg = uut.flow_graph("top")?;
        let mut file = std::fs::File::create("register.dot").unwrap();
        write_dot(&fg, &mut file).unwrap();
        Ok(())
    }

    #[test]
    fn test_compile_times() -> miette::Result<()> {
        let tic = std::time::Instant::now();
        let uut = U::<32, 32>::default();
        let _hdl = uut.flow_graph("top")?;
        let toc = tic.elapsed();
        println!("HDL generation took {:?}", toc);
        Ok(())
    }

    #[test]
    fn test_register_works() -> miette::Result<()> {
        let uut = U::<32, 32>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let io = uut.run(input)?.synchronous_sample();
        let io = io.filter_map(|x| x.value.2.data).collect::<Vec<_>>();
        assert_eq!(io, vec![bits(42), bits(43), bits(42)]);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = U::<1, 1>::default();
        let input = test_stream().stream_after_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &TestBenchOptions::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
