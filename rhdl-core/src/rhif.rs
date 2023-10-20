// RHDL Intermediate Form (RHIF).

use anyhow::Result;

use crate::ast::ExprLit;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // lhs <- arg1 op arg2
    Binary {
        op: AluBinary,
        lhs: Slot,
        arg1: Slot,
        arg2: Slot,
    },
    // lhs <- op arg1
    Unary {
        op: AluUnary,
        lhs: Slot,
        arg1: Slot,
    },
    // return a
    Return(Option<Slot>),
    // lhs <- if cond { then_branch } else { else_branch }
    If {
        lhs: Slot,
        cond: Slot,
        then_branch: BlockId,
        else_branch: BlockId,
    },
    // x <- a[i]
    Index(IndexOp),
    // x <- a
    Copy(CopyOp),
    // *x <- a
    Assign(AssignOp),
    // x <- a.field
    Field(FieldOp),
    // x <- [a; count]
    Repeat(RepeatOp),
    // x <- Struct { fields }
    Struct(StructOp),
    // x <- Tuple(fields)
    Tuple(TupleOp),
    // x = &a
    Ref(RefOp),
    // x = &a.field
    FieldRef(FieldRefOp),
    // x = &a[i]
    IndexRef(IndexRefOp),
    // Jump to block
    Block(BlockId),
    // ROM table
    Case(CaseOp),
    // Exec a function
    Exec(ExecOp),
    // x <- [a, b, c, d]
    Array(ArrayOp),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecOp {
    pub lhs: Slot,
    pub path: Vec<String>,
    pub args: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseOp {
    pub lhs: Slot,
    pub expr: Slot,
    pub table: Vec<(CaseArgument, BlockId)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaseArgument {
    Literal(Slot),
    Wild,
    Path(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyOp {
    pub lhs: Slot,
    pub rhs: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefOp {
    pub lhs: Slot,
    pub arg: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TupleOp {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayOp {
    pub lhs: Slot,
    pub elements: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructOp {
    pub lhs: Slot,
    pub path: Vec<String>,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatOp {
    pub lhs: Slot,
    pub value: Slot,
    pub len: Slot,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FieldOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignOp {
    pub lhs: Slot,
    pub rhs: Slot,
}
#[derive(Debug, Clone, PartialEq)]
pub struct IndexOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfOp {}

#[derive(Debug, Clone, PartialEq)]
pub enum AluBinary {
    Add,
    Sub,
    Mul,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AluUnary {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Slot {
    Literal(usize),
    Register(usize),
    Empty,
}
impl Slot {
    pub fn reg(&self) -> Result<usize> {
        match self {
            Slot::Register(r) => Ok(*r),
            _ => Err(anyhow::anyhow!("Not a register")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);