use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Circuit, CircuitDQ)]
pub struct U<T: Digital + Default, W: Domain, R: Domain, const N: usize> {
    inner: super::asynchronous::U<T, W, R, N>,
}

impl<T: Digital + Default, W: Domain, R: Domain, const N: usize> U<T, W, R, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        Self {
            inner: super::asynchronous::U::new(initial),
        }
    }
}

type ReadI<const N: usize> = super::asynchronous::ReadI<N>;

#[derive(Debug, Digital)]
pub struct WriteI<T: Digital + Default, const N: usize> {
    pub clock: Clock,
    pub data: Option<(Bits<N>, T)>,
}

#[derive(Debug, Digital, Timed)]
pub struct I<T: Digital + Default, W: Domain, R: Domain, const N: usize> {
    pub write: Signal<WriteI<T, N>, W>,
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital + Default, W: Domain, R: Domain, const N: usize> CircuitIO for U<T, W, R, N> {
    type I = I<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = ram_kernel<T, W, R, N>;
}

#[kernel]
pub fn ram_kernel<T: Digital + Default, W: Domain, R: Domain, const N: usize>(
    i: I<T, W, R, N>,
    q: Q<T, W, R, N>,
) -> (Signal<T, R>, D<T, W, R, N>) {
    // We need a struct for the write inputs to the RAM
    let mut w = super::asynchronous::WriteI::<T, N>::dont_care();
    // These are mapped from our input signals
    let i_val = i.write.val();
    w.clock = i_val.clock;
    if let Some((addr, data)) = i_val.data {
        w.data = data;
        w.enable = true;
        w.addr = addr;
    } else {
        w.data = T::default();
        w.enable = false;
        w.addr = bits(0);
    }
    let mut d = D::<T, W, R, N>::dont_care();
    d.inner.write = signal(w);
    d.inner.read = i.read;
    let o = q.inner;
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expect_test::expect;

    use super::*;

    fn get_scan_out_stream<const N: usize>(
        read_clock: u64,
        count: usize,
    ) -> impl Iterator<Item = TimedSample<ReadI<N>>> + Clone {
        let scan_addr = (0..(1 << N)).map(bits::<N>).cycle().take(count);
        let stream_read = scan_addr.stream().clock_pos_edge(read_clock);
        stream_read.map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        })
    }

    fn get_write_stream<T: Digital + Default, const N: usize>(
        write_clock: u64,
        write_data: impl Iterator<Item = Option<(Bits<N>, T)>> + Clone,
    ) -> impl Iterator<Item = TimedSample<WriteI<T, N>>> + Clone {
        let stream_write = write_data.stream().clock_pos_edge(write_clock);
        stream_write.map(|t| {
            t.map(|(cr, val)| WriteI {
                data: val,
                clock: cr.clock,
            })
        })
    }

    #[test]
    fn test_ram_flow_graph() -> miette::Result<()> {
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let fg = uut.flow_graph("uut")?;
        let hdl = fg.hdl("top")?;
        std::fs::write("ram_fg.v", hdl.to_string()).unwrap();
        Ok(())
    }

    #[test]
    fn test_ram_as_verilog() -> miette::Result<()> {
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 34);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat(None).take(50));
        // Stitch the two streams together
        let stream = stream_read.merge(stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });
        let test_bench = uut.run(stream)?.collect::<TestBench<_, _>>();
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_behavior() -> miette::Result<()> {
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits(0))),
        );
        let writes = vec![
            Some((bits(0), bits(142))),
            Some((bits(5), bits(89))),
            Some((bits(2), bits(100))),
            None,
            Some((bits(15), bits(23))),
        ];
        let stream_read = get_scan_out_stream(100, 32);
        let stream_write = get_write_stream(70, writes.into_iter().chain(std::iter::repeat(None)));
        let stream = stream_read.merge(stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });
        let expected = vec![142, 0, 100, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23]
            .into_iter()
            .map(|x| signal(bits(x)));
        let vcd = uut.run(stream.clone())?.collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("option_async");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["b80c662b12a686b5556e8ec4cfcfe6b25b29d1c5459d42af4c4b2cff9e236838"];
        let digest = vcd.dump_to_file(&root.join("ram_write.vcd")).unwrap();
        expect.assert_eq(&digest);
        let output = uut
            .run(stream)?
            .glitch_check(|x| (x.value.0.read.val().clock, x.value.1.val()))
            .sample_at_pos_edge(|x| x.value.0.read.val().clock)
            .skip(17)
            .map(|x| x.value.1);
        let expected = expected.collect::<Vec<_>>();
        let output = output.collect::<Vec<_>>();
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_ram_read_only_behavior() -> miette::Result<()> {
        // Let's start with a simple test where the RAM is pre-initialized,
        // and we just want to read it.
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 32);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat(None).take(50));
        // Stitch the two streams together
        let stream = merge(stream_read, stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });
        let values = (0..16).map(|x| bits(15 - x)).cycle().take(32);
        let samples = uut
            .run(stream)?
            .sample_at_pos_edge(|i| i.value.0.read.val().clock)
            .skip(1);
        let output = samples.map(|x| x.value.1.val());
        assert!(values.eq(output));
        Ok(())
    }
}
