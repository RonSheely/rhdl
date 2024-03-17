use crate::{
    path::{bit_range, Path},
    schematic::{
        components::{
            ArrayComponent, BinaryComponent, BufferComponent, CaseComponent, CastComponent,
            ComponentKind, DigitalFlipFlopComponent, EnumComponent, IndexComponent,
            RepeatComponent, SelectComponent, SpliceComponent, StructComponent, TupleComponent,
            UnaryComponent,
        },
        schematic_impl::{PinPath, Schematic, Trace, WirePath},
    },
};
use anyhow::{ensure, Result};

use super::index::IndexedSchematic;

fn upstream_array(array: &ArrayComponent, output: PinPath) -> Result<Vec<PinPath>> {
    if let Some(upstream) = array
        .elements
        .iter()
        .enumerate()
        .find(|(ndx, &ix)| Path::default().index(*ndx).is_prefix_of(&output.path))
    {
        Ok(vec![PinPath {
            pin: *upstream.1,
            path: output
                .path
                .clone()
                .strip_prefix(&Path::default().index(upstream.0))?,
        }])
    } else {
        Ok(vec![])
    }
}

fn upstream_binary(binary: &BinaryComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![
        PinPath {
            pin: binary.input1,
            path: output.path.clone(),
        },
        PinPath {
            pin: binary.input2,
            path: output.path.clone(),
        },
    ])
}

fn upstream_buffer(buffer: &BufferComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: buffer.input,
        path: output.path,
    }])
}

fn upstream_case(case: &CaseComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(case
        .table
        .iter()
        .map(|(_, ix)| PinPath {
            pin: *ix,
            path: output.path.clone(),
        })
        .collect())
}

fn upstream_dff(dff: &DigitalFlipFlopComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![])
}

fn upstream_enum(e: &EnumComponent, output: PinPath) -> Result<Vec<PinPath>> {
    let discriminant = e.template.discriminant()?.as_i64()?;
    if let Some(field) = e.fields.iter().find(|field| {
        Path::default()
            .payload_by_value(discriminant)
            .field(&field.member.to_string())
            .is_prefix_of(&output.path)
    }) {
        Ok(vec![PinPath {
            pin: field.pin,
            path: output.path.clone().strip_prefix(
                &Path::default()
                    .payload_by_value(discriminant)
                    .field(&field.member.to_string()),
            )?,
        }])
    } else {
        Ok(vec![])
    }
}

fn upstream_index(i: &IndexComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: i.arg,
        path: i.path.clone().join(&output.path),
    }])
}

fn upstream_repeat(r: &RepeatComponent, output: PinPath) -> Result<Vec<PinPath>> {
    if let Some(pin) = (0..r.len).find(|ndx| Path::default().index(*ndx).is_prefix_of(&output.path))
    {
        Ok(vec![PinPath {
            pin: r.value,
            path: output
                .path
                .clone()
                .strip_prefix(&Path::default().index(pin))?,
        }])
    } else {
        Ok(vec![])
    }
}

fn upstream_select(s: &SelectComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![
        PinPath {
            pin: s.true_value,
            path: output.path.clone(),
        },
        PinPath {
            pin: s.false_value,
            path: output.path.clone(),
        },
    ])
}

fn upstream_splice(s: &SpliceComponent, output: PinPath) -> Result<Vec<PinPath>> {
    let (output_bit_range, _) = bit_range(s.kind.clone(), &output.path)?;
    ensure!(
        !output.path.any_dynamic(),
        "Unsupported - dynamic path in splice",
    );
    ensure!(
        !s.path.any_dynamic(),
        "Unsupported - dynamic path in splice",
    );
    let (replace_bit_range, _) = bit_range(s.kind.clone(), &s.path)?;
    let output_path_in_replacement = replace_bit_range.contains(&output_bit_range.start);
    if output_path_in_replacement {
        Ok(vec![PinPath {
            pin: s.subst,
            path: output.path.clone().strip_prefix(&s.path)?,
        }])
    } else {
        Ok(vec![PinPath {
            pin: s.orig,
            path: output.path.clone(),
        }])
    }
}

fn upstream_struct(s: &StructComponent, output: PinPath) -> Result<Vec<PinPath>> {
    if let Some(field) = s.fields.iter().find(|field| {
        Path::default()
            .field(&field.member.to_string())
            .is_prefix_of(&output.path)
    }) {
        Ok(vec![PinPath {
            pin: field.pin,
            path: output
                .path
                .strip_prefix(&Path::default().field(&field.member.to_string()))?,
        }])
    } else if let Some(rest) = s.rest {
        Ok(vec![PinPath {
            pin: rest,
            path: output.path.clone(),
        }])
    } else {
        Ok(vec![])
    }
}

fn upstream_tuple(t: &TupleComponent, output: PinPath) -> Result<Vec<PinPath>> {
    if let Some(field) = t
        .fields
        .iter()
        .enumerate()
        .find(|(ndx, pin)| Path::default().index(*ndx).is_prefix_of(&output.path))
    {
        Ok(vec![PinPath {
            pin: *field.1,
            path: output
                .path
                .clone()
                .strip_prefix(&Path::default().index(field.0))?,
        }])
    } else {
        Ok(vec![])
    }
}

fn upstream_cast(c: &CastComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![])
}

fn upstream_unary(u: &UnaryComponent, output: PinPath) -> Result<Vec<PinPath>> {
    Ok(vec![PinPath {
        pin: u.input,
        path: output.path.clone(),
    }])
}

fn get_upstream_pin_paths(is: &IndexedSchematic, output: PinPath) -> Result<Vec<PinPath>> {
    let pin = is.schematic.pin(output.pin);
    let cix = pin.parent;
    let component = is.schematic.component(cix);
    match &component.kind {
        ComponentKind::Array(array) => upstream_array(array, output),
        ComponentKind::Binary(binary) => upstream_binary(binary, output),
        ComponentKind::BlackBox(_) => Ok(vec![]),
        ComponentKind::Buffer(buffer) => upstream_buffer(buffer, output),
        ComponentKind::Case(case) => upstream_case(case, output),
        ComponentKind::Cast(c) => upstream_cast(c, output),
        ComponentKind::DigitalFlipFlop(dff) => upstream_dff(dff, output),
        ComponentKind::Enum(e) => upstream_enum(e, output),
        ComponentKind::Index(i) => upstream_index(i, output),
        ComponentKind::Kernel(_) => Ok(vec![]),
        ComponentKind::Noop => Ok(vec![]),
        ComponentKind::Repeat(r) => upstream_repeat(r, output),
        ComponentKind::Select(s) => upstream_select(s, output),
        ComponentKind::Splice(s) => upstream_splice(s, output),
        ComponentKind::Struct(s) => upstream_struct(s, output),
        ComponentKind::Tuple(t) => upstream_tuple(t, output),
        ComponentKind::Unary(u) => upstream_unary(u, output),
        ComponentKind::Constant(_) => Ok(vec![]),
    }
}

fn follow_upstream(is: &IndexedSchematic, sink: PinPath, tracks: &mut Vec<WirePath>) -> Result<()> {
    if let Some(parents) = is.index.reverse.get(&sink.pin) {
        for parent in parents {
            tracks.push(WirePath {
                source: *parent,
                dest: sink.pin,
                path: sink.path.clone(),
            });
            let parent_pin_path = PinPath {
                pin: *parent,
                path: sink.path.clone(),
            };
            let upstreams = get_upstream_pin_paths(is, parent_pin_path)?;
            for upstream in upstreams {
                follow_upstream(is, upstream, tracks)?
            }
        }
    }
    Ok(())
}

pub fn follow_pin_upstream(schematic: Schematic, pin_path: PinPath) -> Result<Trace> {
    let is: IndexedSchematic = schematic.into();
    let mut paths = vec![];
    follow_upstream(&is, pin_path, &mut paths)?;
    Ok(paths)
}
