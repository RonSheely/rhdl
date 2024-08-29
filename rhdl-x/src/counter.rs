use crate::dff;
use petgraph::{algo::is_cyclic_directed, visit::GraphProp};
use rhdl::{
    core::{
        build_rtl_flow_graph, build_synchronous_flow_graph, compiler::codegen::compile_to_rtl,
        flow_graph::dot::write_dot,
    },
    prelude::*,
};
use std::io::{stderr, Write};

mod comb_adder {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Default, Synchronous)]
    #[rhdl(kernel=adder::<{N}>)]
    pub struct U<const N: usize> {}

    impl<const N: usize> SynchronousIO for U<N> {
        type I = (Bits<N>, Bits<N>);
        type O = Bits<N>;
    }

    impl<const N: usize> SynchronousDQ for U<N> {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn adder<const N: usize>(reset: bool, i: (Bits<N>, Bits<N>), q: ()) -> (Bits<N>, ()) {
        let a = i;
        (a.0 + a.1, ())
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=counter::<{N}>)]
#[rhdl(auto_dq)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
    adder: comb_adder::U<{ N }>,
}

impl<const N: usize> U<N> {
    pub fn new() -> Self {
        Self {
            count: dff::U::new(Bits::ZERO),
            adder: Default::default(),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I;
    type O = Bits<N>;
}

#[kernel]
pub fn counter<const N: usize>(reset: bool, i: I, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if i.enable { q.adder } else { q.count };
    let output = q.count;
    if reset {
        (
            bits(0),
            D::<{ N }> {
                count: bits(0),
                adder: (bits(0), bits(0)),
            },
        )
    } else {
        (
            output,
            D::<{ N }> {
                count: next_count,
                adder: (q.count, bits(1)),
            },
        )
    }
}

#[test]
fn test_counter_timing_root() -> miette::Result<()> {
    use core::hash::Hasher;
    let uut: U<4> = U::new();
    let uut_module = compile_design::<<U<4> as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let rtl = compile_to_rtl(&uut_module)?;
    eprintln!("rtl: {:?}", rtl);
    let fg = build_rtl_flow_graph(&rtl);
    let mut dot = std::fs::File::create("counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    let counter_uut = build_synchronous_flow_graph(&uut.descriptor()?)?;
    let mut dot = vec![0_u8; 0];
    write_dot(&counter_uut, &mut dot).unwrap();
    let mut hasher = fnv::FnvHasher::default();
    hasher.write(&dot);
    let hash = hasher.finish();
    eprintln!("Dot hash: {:x}", hash);
    let mut dot = std::fs::File::create(format!("counter_{hash:x}.dot")).unwrap();
    write_dot(&counter_uut, &mut dot).unwrap();
    assert!(!is_cyclic_directed(&counter_uut.graph));
    eprintln!("rtl: {:?}", rtl);
    let uut = comb_adder::U::<4>::default();
    type Adder = comb_adder::U<4>;
    let uut = compile_design::<<Adder as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let rtl_adder = compile_to_rtl(&uut)?;
    eprintln!("rtl: {:?}", rtl_adder);
    eprintln!("******************************");
    eprintln!("rtl: {:?}", rtl);
    eprintln!("rhif: {:?}", uut_module);
    Ok(())
}

/*
The function to compute timing needs to look something like this:

fn timing(path: &Path, computer: &'dyn CostEstimator) -> Result<CostGraph, RHDLError> {
    let module = compile_design::<<Self as Synchronous>::Update>(CompilationMode::Synchronous)?;
    let top = &module.objects[&module.top];
    let timing = compute_timing_graph(&module, module.top, path, computer)?;
    // Check for inputs to the timing graph that come via the Q path
    let target_argument = top.arguments[2]; // The arguments are Reset, Input, Q
    for input in timing.inputs {
        if input.slot == target_argument {
            if Path::default().field("child1").is_prefix_of(&input.path) {
                let path = path.strip_prefix("child1");
                let child_timing = <Child1 as Synchronous>::timing(&path, computer)?
            }
        }
    }
}

*/
