use expect_test::expect;
use rhdl::prelude::*;
use rhdl_fpga::core::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    reg: dff::U<b8>,
}

impl SynchronousIO for U {
    type I = Option<(bool, b8)>;
    type O = bool;
    type Kernel = test_kernel;
}

#[kernel]
pub fn test_kernel(_cr: ClockReset, i: Option<(bool, b8)>, q: Q) -> (bool, D) {
    let mut d = D::dont_care();
    d.reg = q.reg;
    let mut o = false;
    if let Some((_x, y)) = i {
        d.reg = y;
        o = y.any();
    }
    (o, d)
}

fn stream() -> impl Iterator<Item = TimedSample<(ClockReset, Option<(bool, b8)>)>> {
    vec![
        None,
        Some((false, b8(3))),
        Some((true, b8(1))),
        None,
        Some((false, b8(0))),
        Some((true, b8(5))),
        None,
    ]
    .into_iter()
    .stream_after_reset(1)
    .clock_pos_edge(100)
}

#[test]
fn test_trace() -> miette::Result<()> {
    let uut = U::default();
    let input = stream();
    let vcd = uut.run(input)?.collect::<Vcd>();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vcd")
        .join("flow_graph_if_let");
    std::fs::create_dir_all(&root).unwrap();
    let expect = expect!["0aa56bb1666358898ac118949d1e288404069aaaadb343ccbf5854a0992ba9a8"];
    let digest = vcd
        .dump_to_file(&root.join("flow_graph_if_let.vcd"))
        .unwrap();
    expect.assert_eq(&digest);
    Ok(())
}

#[test]
fn test_hdl() -> miette::Result<()> {
    let uut = U::default();
    let input = stream();
    let tb = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
    let tm = tb.rtl(&uut, &Default::default())?;
    tm.run_iverilog()?;
    let tm = tb.flow_graph(&uut, &Default::default())?;
    tm.run_iverilog()?;
    Ok(())
}
