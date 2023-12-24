use std::ops::Range;

use anyhow::bail;
use anyhow::Result;

use crate::DiscriminantAlignment;
use crate::Kind;

#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    All,
    Index(usize),
    Field(&'static str),
    EnumDiscriminant,
    EnumPayload(&'static str),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    pub elements: Vec<PathElement>,
}

impl Path {
    pub fn index(self, index: usize) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::Index(index));
        Path { elements }
    }
    pub fn field(self, field: &'static str) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::Field(field));
        Path { elements }
    }
    pub fn discriminant(self) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::EnumDiscriminant);
        Path { elements }
    }
    pub fn payload(self, name: &'static str) -> Self {
        let mut elements = self.elements;
        elements.push(PathElement::EnumPayload(name));
        Path { elements }
    }
}

// Given a Kind and a Vec<Path>, compute the bit offsets of
// the endpoint of the path within the original data structure.
pub fn bit_range(kind: Kind, path: &Path) -> Result<(Range<usize>, Kind)> {
    let mut range = 0..kind.bits();
    let mut kind = kind;
    for p in &path.elements {
        match p {
            PathElement::All => (),
            PathElement::Index(i) => match &kind {
                Kind::Array(array) => {
                    let element_size = array.base.bits();
                    if i >= &array.size {
                        bail!("Array index out of bounds")
                    }
                    range = range.start + i * element_size..range.start + (i + 1) * element_size;
                    kind = *array.base.clone();
                }
                Kind::Tuple(tuple) => {
                    if i >= &tuple.elements.len() {
                        bail!("Tuple index out of bounds")
                    }
                    let offset = dbg!(tuple.elements[0..*i]
                        .iter()
                        .map(|e| e.bits())
                        .sum::<usize>());
                    let size = dbg!(tuple.elements[*i].bits());
                    range = range.start + offset..range.start + offset + size;
                    kind = tuple.elements[*i].clone();
                }
                _ => bail!("Indexing non-indexable type"),
            },
            PathElement::Field(field) => match &kind {
                Kind::Struct(structure) => {
                    if !structure.fields.iter().any(|f| &f.name == field) {
                        bail!("Field not found")
                    }
                    let offset = structure
                        .fields
                        .iter()
                        .take_while(|f| &f.name != field)
                        .map(|f| f.kind.bits())
                        .sum::<usize>();
                    let field = &structure
                        .fields
                        .iter()
                        .find(|f| &f.name == field)
                        .unwrap()
                        .kind;
                    let size = field.bits();
                    range = range.start + offset..range.start + offset + size;
                    kind = field.clone();
                }
                _ => bail!("Field indexing not allowed on this type"),
            },
            PathElement::EnumDiscriminant => match &kind {
                Kind::Enum(enumerate) => {
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start..range.start + enumerate.discriminant_layout.width
                        }
                        DiscriminantAlignment::Msb => {
                            range.end - enumerate.discriminant_layout.width..range.end
                        }
                    };
                    kind = if enumerate.discriminant_layout.ty == crate::DiscriminantType::Signed {
                        Kind::make_signed(enumerate.discriminant_layout.width)
                    } else {
                        Kind::make_bits(enumerate.discriminant_layout.width)
                    };
                }
                _ => bail!("Enum discriminant not valid for non-enum types"),
            },
            PathElement::EnumPayload(name) => match &kind {
                Kind::Enum(enumerate) => {
                    let field = enumerate
                        .variants
                        .iter()
                        .find(|f| &f.name == name)
                        .ok_or_else(|| anyhow::anyhow!("Enum payload not found"))?
                        .kind
                        .clone();
                    range = match enumerate.discriminant_layout.alignment {
                        DiscriminantAlignment::Lsb => {
                            range.start + enumerate.discriminant_layout.width
                                ..range.start + enumerate.discriminant_layout.width + field.bits()
                        }
                        DiscriminantAlignment::Msb => range.start..range.start + field.bits(),
                    };
                    dbg!(&range);
                    kind = field;
                }
                _ => bail!("Enum payload not valid for non-enum types"),
            },
        }
    }
    Ok((range, kind))
}
