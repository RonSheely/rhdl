use std::fmt::Display;

use crate::{
    rhif::{AluBinary, AluUnary, BlockId, CaseArgument, FieldValue, FuncId, Member, OpCode, Slot},
    util::splice,
};

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Binary {
                op,
                lhs,
                arg1,
                arg2,
            } => {
                write!(f, " {} <- {} {} {}", lhs, arg1, op, arg2)
            }
            OpCode::Unary { op, lhs, arg1 } => {
                write!(f, " {} <- {}{}", lhs, op, arg1)
            }
            OpCode::Array { lhs, elements } => {
                write!(f, " {} <- [{}]", lhs, splice(elements, ", "))
            }
            OpCode::Assign { lhs, rhs } => {
                write!(f, "*{} <- {}", lhs, rhs)
            }
            OpCode::Ref { lhs, arg } => write!(f, " {} <- &{}", lhs, arg),
            OpCode::IndexRef { lhs, arg, index } => write!(f, " {} <- &{}[{}]", lhs, arg, index),
            OpCode::FieldRef { lhs, arg, member } => write!(f, " {} <- &{}.{}", lhs, arg, member),
            OpCode::If {
                lhs,
                cond,
                then_branch,
                else_branch,
            } => {
                write!(
                    f,
                    " {} <- if {} then {} else {}",
                    lhs, cond, then_branch, else_branch
                )
            }
            OpCode::Return { result } => {
                if let Some(result) = result {
                    write!(f, " ret {}", result)
                } else {
                    write!(f, " ret")
                }
            }
            OpCode::Copy { lhs, rhs } => {
                write!(f, " {} <- {}", lhs, rhs)
            }
            OpCode::Tuple { lhs, fields } => {
                write!(f, " {} <- ({})", lhs, splice(fields, ", "))
            }
            OpCode::Field { lhs, arg, member } => {
                write!(f, " {} <- {}.{}", lhs, arg, member)
            }
            OpCode::Index { lhs, arg, index } => {
                write!(f, " {} <- {}[{}]", lhs, arg, index)
            }
            OpCode::Case {
                discriminant: expr,
                table,
            } => {
                writeln!(f, " case {}", expr)?;
                for (cond, val) in table {
                    writeln!(f, "         {} => {}", cond, val)?;
                }
                Ok(())
            }
            OpCode::Exec { lhs, id, args } => {
                write!(f, " {} <- {}({})", lhs, id, splice(args, ", "))
            }
            OpCode::Struct {
                lhs,
                path,
                fields,
                rest,
            } => {
                write!(
                    f,
                    " {} <- {} {{ {} {} }}",
                    lhs,
                    path,
                    splice(fields, ", "),
                    rest.map(|x| format!("..{}", x)).unwrap_or_default()
                )
            }
            OpCode::Repeat { lhs, value, len } => {
                write!(f, " {} <- [{}; {}]", lhs, value, len)
            }
            OpCode::Block(BlockId(x)) => write!(f, " sub B{x}"),
            OpCode::Comment(s) => write!(f, " # {}", s.trim_end().replace('\n', "\n   # ")),
            OpCode::Payload {
                lhs,
                arg,
                discriminant,
            } => {
                write!(f, " {} <- {}#[{}]", lhs, arg, discriminant)
            }
            OpCode::Discriminant { lhs, arg } => write!(f, " {} <- #[{}]", lhs, arg),
            OpCode::Enum {
                lhs,
                path,
                discriminant,
                fields,
            } => {
                write!(
                    f,
                    " {} <- {}#{}({})",
                    lhs,
                    path,
                    discriminant,
                    splice(fields, ", ")
                )
            }
            OpCode::AsBits { lhs, arg, len } => {
                write!(f, " {} <- {} as b{}", lhs, arg, len)
            }
            OpCode::AsSigned { lhs, arg, len } => {
                write!(f, " {} <- {} as s{}", lhs, arg, len)
            }
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.member, self.value)
    }
}

impl Display for CaseArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseArgument::Literal(l) => write!(f, "{}", l),
            CaseArgument::Wild => write!(f, "_"),
        }
    }
}

impl Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl Display for FuncId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "f{}", self.0)
    }
}

impl Display for Member {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Member::Named(s) => write!(f, "{}", s),
            Member::Unnamed(i) => write!(f, "{}", i),
        }
    }
}

impl Display for AluBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluBinary::Add => write!(f, "+"),
            AluBinary::Sub => write!(f, "-"),
            AluBinary::Mul => write!(f, "*"),
            AluBinary::BitAnd => write!(f, "&"),
            AluBinary::BitOr => write!(f, "|"),
            AluBinary::BitXor => write!(f, "^"),
            AluBinary::Shl => write!(f, "<<"),
            AluBinary::Shr => write!(f, ">>"),
            AluBinary::Eq => write!(f, "=="),
            AluBinary::Ne => write!(f, "!="),
            AluBinary::Lt => write!(f, "<"),
            AluBinary::Le => write!(f, "<="),
            AluBinary::Gt => write!(f, ">"),
            AluBinary::Ge => write!(f, ">="),
            AluBinary::And => write!(f, "&&"),
            AluBinary::Or => write!(f, "||"),
        }
    }
}

impl Display for AluUnary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AluUnary::Neg => write!(f, "-"),
            AluUnary::Not => write!(f, "!"),
            AluUnary::All => write!(f, "&"),
            AluUnary::Any => write!(f, "|"),
            AluUnary::Xor => write!(f, "^"),
            AluUnary::Signed => write!(f, "signed "),
            AluUnary::Unsigned => write!(f, "unsigned "),
        }
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Literal(l) => write!(f, "l{}", l),
            // Use 4 spaces for alignment
            Slot::Register(usize) => write!(f, "r{}", usize),
            Slot::Empty => write!(f, "{{}}"),
        }
    }
}
