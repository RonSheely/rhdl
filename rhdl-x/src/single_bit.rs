use rhdl::prelude::*;

use crate::dff;

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=single_bit)]
#[rhdl(auto_dq)]
pub struct U {
    state: dff::U<bool>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            state: dff::U::new(false),
        }
    }
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
}

#[kernel]
pub fn single_bit(reset: bool, i: bool, q: Q) -> (bool, D) {
    let next_state = if i { !q.state } else { q.state };
    let output = q.state;
    if reset {
        (false, D { state: false })
    } else {
        (output, D { state: next_state })
    }
}

#[test]
fn test_single_bit() -> miette::Result<()> {
    let uut = U::default();
    let module = compile_design::<single_bit>(CompilationMode::Synchronous)?;
    let rtl = compile_to_rtl(&module)?;
    let uut_fg = build_synchronous_flow_graph(&uut.descriptor()?);
    let mut dot = std::fs::File::create("single_bit.dot").unwrap();
    write_dot(&uut_fg, &mut dot).unwrap();
    eprintln!("************* RTL *************");
    eprintln!("RTL {:?}", rtl);
    Ok(())
}
