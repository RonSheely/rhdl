use crate::{
    ast::source_location::SourceLocation,
    rhif::spec::{AluBinary, AluUnary},
    types::bit_string::BitString,
};

#[derive(Debug, Clone, Hash)]
pub struct Binary {
    pub op: AluBinary,
}

#[derive(Debug, Clone, Hash)]
pub struct Unary {
    pub op: AluUnary,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicIndex {
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct DynamicSplice {
    pub len: usize,
}

#[derive(Debug, Clone, Hash)]
pub struct BlackBox {
    pub name: String,
}

#[derive(Debug, Clone, Hash)]
pub struct Case {
    pub entries: Vec<CaseEntry>,
}

#[derive(Debug, Clone, Hash)]
pub enum CaseEntry {
    Literal(BitString),
    WildCard,
}

#[derive(Debug, Clone, Hash)]
pub enum ComponentKind {
    Binary(Binary),
    BlackBox(BlackBox),
    Buffer(String),
    Case(Case),
    Constant(bool),
    DFFInput(DFFInput),
    DFFOutput(DFFOutput),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Input(Input),
    Output(Output),
    Select,
    TimingStart,
    TimingEnd,
    Unary(Unary),
}

#[derive(Debug, Clone, Hash)]
pub struct Input {
    pub argument_index: usize,
    pub bit_index: usize,
    pub name: String,
}

#[derive(Debug, Clone, Hash)]
pub struct Output {
    pub bit_index: usize,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DFFInput {
    pub bit_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DFFOutput {
    pub bit_index: usize,
}

#[derive(Clone, Hash)]
pub struct Component {
    pub kind: ComponentKind,
    pub width: usize,
    pub location: Option<SourceLocation>,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ComponentKind::Binary(binary) => write!(f, "{:?}", binary.op),
            ComponentKind::BlackBox(blackbox) => write!(f, "{}", blackbox.name),
            ComponentKind::Buffer(name) => write!(f, "{name}"),
            ComponentKind::Case(_) => write!(f, "Case"),
            ComponentKind::Constant(constant) => {
                write!(f, "const({})", if *constant { 1 } else { 0 })
            }
            ComponentKind::DynamicIndex(dynamic_index) => write!(f, "[[{}]]", dynamic_index.len),
            ComponentKind::DynamicSplice(dynamic_splice) => write!(f, "//{}//", dynamic_splice.len),
            ComponentKind::Input(input) => {
                write!(
                    f,
                    "[{}]<-in<{}, {}>",
                    input.name, input.argument_index, input.bit_index
                )
            }
            ComponentKind::Output(output) => {
                write!(f, "[{}]->out<{}>", output.name, output.bit_index)
            }
            ComponentKind::Select => write!(f, "?"),
            ComponentKind::DFFInput(dff_input) => {
                write!(f, "dff_in[{}]", dff_input.bit_index)
            }
            ComponentKind::DFFOutput(dff_output) => {
                write!(f, "dff_out[{}]", dff_output.bit_index)
            }
            ComponentKind::TimingStart => write!(f, "timing_start"),
            ComponentKind::TimingEnd => write!(f, "timing_end"),
            ComponentKind::Unary(unary) => write!(f, "{:?}", unary.op),
        }?;
        writeln!(f)
    }
}
