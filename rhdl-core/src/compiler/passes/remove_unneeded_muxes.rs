use crate::{
    error::RHDLError,
    rhif::{
        spec::{Assign, OpCode},
        Object,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveUnneededMuxesPass {}

impl Pass for RemoveUnneededMuxesPass {
    fn name() -> &'static str {
        "remove_unneeded_muxes"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        for lop in input.ops.iter_mut() {
            if let OpCode::Select(select) = lop.op.clone() {
                if let Ok(literal) = select.cond.as_literal() {
                    let val = &input.literals[&literal];
                    if val.as_bool()? {
                        lop.op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.true_value,
                        });
                    } else {
                        lop.op = OpCode::Assign(Assign {
                            lhs: select.lhs,
                            rhs: select.false_value,
                        });
                    }
                } else if select.true_value == select.false_value {
                    lop.op = OpCode::Assign(Assign {
                        lhs: select.lhs,
                        rhs: select.true_value,
                    });
                }
            }
        }
        Ok(input)
    }
}
