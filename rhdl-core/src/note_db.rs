use crate::types::note::Notable;
use crate::{NoteKey, NoteWriter};
use anyhow::bail;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::{cell::RefCell, hash::Hasher, io::Write};
use vcd::{IdCode, VarType};

struct TimeSeries<T> {
    values: Vec<(u64, T)>,
    width: u8,
}

impl<T> TimeSeries<T> {
    fn new(time: u64, value: T, width: u8) -> Self {
        Self {
            values: vec![(time, value)],
            width,
        }
    }
    fn cursor<W: Write>(
        &self,
        details: &TimeSeriesDetails,
        name: &str,
        writer: &mut vcd::Writer<W>,
    ) -> Option<Cursor> {
        let name_sanitized = name.replace("::", "__");
        let code = if self.width != 0 {
            writer.add_wire(self.width as u32, &name_sanitized).ok()?
        } else {
            writer
                .add_var(VarType::String, 0, &name_sanitized, None)
                .ok()?
        };
        self.values.first().map(|x| Cursor {
            kind: details.kind,
            next_time: Some(x.0),
            hash: details.hash,
            ptr: 0,
            code,
            code_as_bytes: code.to_string().into_bytes(),
        })
    }
    fn advance_cursor(&self, cursor: &mut Cursor) {
        cursor.ptr += 1;
        if let Some((time, _)) = self.values.get(cursor.ptr) {
            cursor.next_time = Some(*time);
        } else {
            cursor.next_time = None;
        }
    }
}

impl TimeSeries<bool> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            writer
                .writer()
                .write_all(if *value { b"1" } else { b"0" })?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<u128> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            bits_to_vcd(*value, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<i128> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            bits_to_vcd(*value as u128, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<&'static str> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            writer.change_string(cursor.code, value)?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Tristate {
    value: u128,
    mask: u128,
}

impl TimeSeries<Tristate> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            tristate_to_vcd(value.value, value.mask, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl<T: PartialEq> TimeSeries<T> {
    fn push(&mut self, time: u64, value: T, width: u8) {
        if let Some((_last_time, last_value)) = self.values.last() {
            if *last_value == value {
                return;
            }
        }
        self.values.push((time, value));
        assert_eq!(self.width, width);
    }
}

type TimeSeriesHash = u32;

struct TimeSeriesDetails {
    kind: TimeSeriesKind,
    hash: TimeSeriesHash,
    path: Vec<&'static str>,
    key: String,
}

fn tristate_to_vcd(x: u128, mask: u128, width: usize, buffer: &mut [u8]) {
    (0..width).for_each(|i| {
        buffer[i] = if mask & (1 << (width - 1 - i)) != 0 {
            if x & (1 << (width - 1 - i)) != 0 {
                b'1'
            } else {
                b'0'
            }
        } else {
            b'z'
        };
    })
}

fn bits_to_vcd(x: u128, width: usize, buffer: &mut [u8]) {
    (0..width).for_each(|i| {
        buffer[i] = if x & (1 << (width - 1 - i)) != 0 {
            b'1'
        } else {
            b'0'
        };
    })
}

#[derive(Default)]
pub struct NoteDB {
    db_bool: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<bool>>,
    db_bits: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<u128>>,
    db_signed: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<i128>>,
    db_string: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<&'static str>>,
    db_tristate: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<Tristate>>,
    details: fnv::FnvHashMap<TimeSeriesHash, TimeSeriesDetails>,
    path: Vec<&'static str>,
    time: u64,
}

struct Cursor {
    next_time: Option<u64>,
    hash: TimeSeriesHash,
    kind: TimeSeriesKind,
    ptr: usize,
    code: IdCode,
    code_as_bytes: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
enum TimeSeriesKind {
    Bool,
    Bits,
    Signed,
    String,
    Tristate,
}

impl NoteWriter for NoteDB {
    fn write_bool(&mut self, key: impl NoteKey, value: bool) {
        self.note_bool(key, value);
    }

    fn write_bits(&mut self, key: impl NoteKey, value: u128, len: u8) {
        self.note_u128(key, value, len);
    }

    fn write_signed(&mut self, key: impl NoteKey, value: i128, len: u8) {
        self.note_i128(key, value, len);
    }

    fn write_string(&mut self, key: impl NoteKey, value: &'static str) {
        self.note_string(key, value);
    }

    fn write_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, size: u8) {
        self.note_tristate(key, value, mask, size);
    }
}

impl NoteDB {
    fn push_path(&mut self, name: &'static str) {
        self.path.push(name);
    }
    fn pop_path(&mut self) {
        self.path.pop();
    }
    fn define_new_time_series(
        &mut self,
        key: &impl NoteKey,
        kind: TimeSeriesKind,
        key_hash: TimeSeriesHash,
    ) {
        eprintln!(
            "Defining new time series: {path:?} {key:?} {kind:?}",
            path = self.path,
            key = key.as_string(),
            kind = kind
        );
        self.details.insert(
            key_hash,
            TimeSeriesDetails {
                kind,
                hash: key_hash,
                path: self.path.clone(),
                key: key.as_string().to_string(),
            },
        );
    }
    fn key_hash(&self, key: &impl NoteKey) -> TimeSeriesHash {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        hasher.finish() as TimeSeriesHash
    }
    fn note_bool(&mut self, key: impl NoteKey, value: bool) {
        let key_hash = self.key_hash(&key);
        if let Some(values) = self.db_bool.get_mut(&key_hash) {
            values.push(self.time, value, 1);
        } else {
            self.define_new_time_series(&key, TimeSeriesKind::Bool, key_hash);
            self.db_bool
                .insert(key_hash, TimeSeries::new(self.time, value, 1));
        }
    }
    fn note_u128(&mut self, key: impl NoteKey, value: u128, width: u8) {
        let key_hash = self.key_hash(&key);
        if let Some(values) = self.db_bits.get_mut(&key_hash) {
            values.push(self.time, value, width);
        } else {
            self.define_new_time_series(&key, TimeSeriesKind::Bits, key_hash);
            self.db_bits
                .insert(key_hash, TimeSeries::new(self.time, value, width));
        }
    }
    fn note_i128(&mut self, key: impl NoteKey, value: i128, width: u8) {
        let key_hash = self.key_hash(&key);
        if let Some(values) = self.db_signed.get_mut(&key_hash) {
            values.push(self.time, value, width);
        } else {
            self.define_new_time_series(&key, TimeSeriesKind::Signed, key_hash);
            self.db_signed
                .insert(key_hash, TimeSeries::new(self.time, value, width));
        }
    }
    fn note_string(&mut self, key: impl NoteKey, value: &'static str) {
        let key_hash = self.key_hash(&key);
        if let Some(values) = self.db_string.get_mut(&key_hash) {
            values.push(self.time, value, 0);
        } else {
            self.define_new_time_series(&key, TimeSeriesKind::String, key_hash);
            self.db_string
                .insert(key_hash, TimeSeries::new(self.time, value, 0));
        }
    }
    fn note_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, width: u8) {
        let key_hash = self.key_hash(&key);
        if let Some(values) = self.db_tristate.get_mut(&key_hash) {
            values.push(self.time, Tristate { value, mask }, width);
        } else {
            self.define_new_time_series(&key, TimeSeriesKind::Tristate, key_hash);
            self.db_tristate.insert(
                key_hash,
                TimeSeries::new(self.time, Tristate { value, mask }, width),
            );
        }
    }

    fn setup_cursor<W: Write>(
        &self,
        name: &str,
        details: &TimeSeriesDetails,
        writer: &mut vcd::Writer<W>,
    ) -> Option<Cursor> {
        match details.kind {
            TimeSeriesKind::Bits => self
                .db_bits
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Bool => self
                .db_bool
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Signed => self
                .db_signed
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::String => self
                .db_string
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Tristate => self
                .db_tristate
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
        }
    }
    fn write_advance_cursor<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        match cursor.kind {
            TimeSeriesKind::Bits => self
                .db_bits
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Bool => self
                .db_bool
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Signed => self
                .db_signed
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::String => self
                .db_string
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Tristate => self
                .db_tristate
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
        }
    }
    fn setup_cursors<W: Write>(
        &self,
        name: &str,
        scope: &Scope,
        cursors: &mut Vec<Cursor>,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        writer.add_module(name)?;
        for (name, hash) in &scope.signals {
            let details = self.details.get(hash).unwrap();
            if let Some(cursor) = self.setup_cursor(name, details, writer) {
                cursors.push(cursor);
            }
        }
        for (name, child) in &scope.children {
            self.setup_cursors(name, child, cursors, writer)?;
        }
        writer.upscope()?;
        Ok(())
    }
    pub fn dump_vcd<W: Write>(&self, w: W) -> anyhow::Result<()> {
        let mut writer = vcd::Writer::new(w);
        writer.timescale(1, vcd::TimescaleUnit::PS)?;
        let root_scope = hierarchical_walk(self.details.iter().map(|(hash, details)| TSItem {
            path: &details.path,
            name: &details.key,
            hash: *hash,
        }));
        let mut cursors = vec![];
        self.setup_cursors("top", &root_scope, &mut cursors, &mut writer)?;
        writer.enddefinitions()?;
        writer.timestamp(0)?;
        let mut current_time = 0;
        let mut keep_running = true;
        while keep_running {
            keep_running = false;
            let mut next_time = !0;
            let mut found_match = true;
            while found_match {
                found_match = false;
                for cursor in &mut cursors {
                    if cursor.next_time == Some(current_time) {
                        self.write_advance_cursor(cursor, &mut writer)?;
                        found_match = true;
                    } else if let Some(time) = cursor.next_time {
                        next_time = next_time.min(time);
                    }
                    if cursor.next_time.is_some() {
                        keep_running = true;
                    }
                }
            }
            if next_time != !0 {
                current_time = next_time;
                writer.timestamp(current_time)?;
            }
        }
        Ok(())
    }
}

thread_local! {
    static DB: RefCell<Option<NoteDB>> = const { RefCell::new(None) };
}

// This is not send or sync because it's not safe to share across threads.
pub struct NoteDBGuard {}

impl NoteDBGuard {
    pub fn take(self) -> NoteDB {
        let opt = DB.with(|db| db.borrow_mut().take());
        opt.unwrap_or_default()
    }
}

impl Drop for NoteDBGuard {
    fn drop(&mut self) {
        DB.with(|db| {
            let mut db = db.borrow_mut();
            *db = None;
        });
    }
}

#[must_use]
pub fn note_init_db() -> NoteDBGuard {
    DB.replace(Some(NoteDB::default()));
    NoteDBGuard {}
}

pub fn with_note_db<F: FnMut(&NoteDB)>(mut f: F) {
    DB.with(|db| {
        let db = db.borrow();
        if let Some(db) = db.as_ref() {
            f(db)
        }
    });
}

pub fn note_push_path(name: &'static str) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.push_path(name)
        }
    });
}

pub fn note_pop_path() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.pop_path()
        }
    });
}

pub fn note_time(time: u64) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.time = time
        }
    });
}

pub fn note(key: impl NoteKey, value: impl Notable) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            value.note(key, db)
        }
    });
}

// Every item has a name.  This is either the name of the scope or the signal
// Scopes can contain other scopes or signals.
// Signals are terminal (and connect to a hash)
// The top level thing is a scope.

#[derive(Default)]
struct Scope {
    children: BTreeMap<&'static str, Box<Scope>>,
    signals: BTreeMap<String, TimeSeriesHash>,
}

struct TSItem<'a> {
    path: &'a [&'static str],
    name: &'a str,
    hash: TimeSeriesHash,
}

fn hierarchical_walk<'a>(paths: impl Iterator<Item = TSItem<'a>>) -> Scope {
    let mut root = Scope::default();
    for ts_item in paths {
        let mut folder = &mut root;
        for item in ts_item.path {
            if !folder.children.contains_key(item) {
                let new_folder = Box::new(Scope::default());
                folder.children.insert(item, new_folder);
            }
            folder = folder.children.get_mut(item).unwrap();
        }
        folder.signals.insert(ts_item.name.into(), ts_item.hash);
    }
    root
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use rhdl_bits::Bits;

    use crate::{types::kind::Variant, Digital, DiscriminantAlignment, Kind};

    use super::*;

    #[test]
    fn test_vcd_write() {
        let guard = note_init_db();
        for i in 0..1000 {
            note_time(i * 1000);
            note("a", i % 2 == 0);
            note("b", i % 2 == 1);
        }
        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd).unwrap();
        std::fs::write("test.vcd", vcd).unwrap();
    }

    #[test]
    fn test_vcd_with_enum() {
        #[derive(Copy, Clone, PartialEq, Default)]
        enum Mixed {
            #[default]
            None,
            Bool(bool),
            Tuple(bool, Bits<3>),
            Array([bool; 3]),
            Strct {
                a: bool,
                b: Bits<3>,
            },
        }

        impl Digital for Mixed {
            const BITS: usize = 7;
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "Mixed",
                    vec![
                        Variant {
                            name: "None".to_string(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Bool".to_string(),
                            discriminant: 1,
                            kind: Kind::make_bits(1),
                        },
                        Variant {
                            name: "Tuple".to_string(),
                            discriminant: 2,
                            kind: Kind::make_tuple(vec![Kind::make_bits(1), Kind::make_bits(3)]),
                        },
                        Variant {
                            name: "Array".to_string(),
                            discriminant: 3,
                            kind: Kind::make_array(Kind::make_bits(1), 3),
                        },
                        Variant {
                            name: "Strct".to_string(),
                            discriminant: 4,
                            kind: Kind::make_struct(
                                "Mixed::Strct",
                                vec![
                                    Kind::make_field("a", Kind::make_bits(1)),
                                    Kind::make_field("b", Kind::make_bits(3)),
                                ],
                            ),
                        },
                    ],
                    Kind::make_discriminant_layout(
                        3,
                        DiscriminantAlignment::Lsb,
                        crate::types::kind::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Vec<bool> {
                let raw = match self {
                    Self::None => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Bool(b) => {
                        let mut v = rhdl_bits::bits::<3>(1).to_bools();
                        v.extend(b.bin());
                        v
                    }
                    Self::Tuple(b, c) => {
                        let mut v = rhdl_bits::bits::<3>(2).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v
                    }
                    Self::Array([b, c, d]) => {
                        let mut v = rhdl_bits::bits::<3>(3).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.extend(d.bin());
                        v
                    }
                    Self::Strct { a, b } => {
                        let mut v = rhdl_bits::bits::<3>(4).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                };
                if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter().chain(repeat(false).take(missing)).collect()
                } else {
                    raw
                }
            }
            fn init() -> Self {
                <Self as Default>::default()
            }
        }

        impl Notable for Mixed {
            fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
                match self {
                    Self::None => {
                        writer.write_string(key, stringify!(None));
                    }
                    Self::Bool(b) => {
                        writer.write_string(key, stringify!(Bool));
                        Notable::note(b, (key, 0), &mut writer);
                    }
                    Self::Tuple(b, c) => {
                        writer.write_string(key, stringify!(Tuple));
                        b.note((key, "b"), &mut writer);
                        c.note((key, "c"), &mut writer);
                    }
                    Self::Array([b, c, d]) => {
                        writer.write_string(key, stringify!(Array));
                        b.note((key, 0), &mut writer);
                        c.note((key, 1), &mut writer);
                        d.note((key, 2), &mut writer);
                    }
                    Self::Strct { a, b } => {
                        writer.write_string(key, stringify!(Strct));
                        a.note((key, "a"), &mut writer);
                        b.note((key, "b"), &mut writer);
                    }
                }
            }
        }

        assert_eq!(Mixed::None.kind().bits(), Mixed::BITS);

        let guard = note_init_db();
        note_time(0);
        note("a", Mixed::None);
        note_time(100);
        note("a", Mixed::Array([true, false, true]));
        note_time(200);
        note(
            "a",
            Mixed::Strct {
                a: true,
                b: rhdl_bits::bits(5),
            },
        );
        note_time(300);
        note("a", Mixed::Bool(false));
        note_time(400);
        note("a", Mixed::Tuple(true, rhdl_bits::bits(3)));
        note_time(500);

        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd).unwrap();
        std::fs::write("test_enum.vcd", vcd).unwrap();
    }

    #[test]
    fn test_vcd_with_nested_paths() {
        let guard = note_init_db();
        for i in 0..10 {
            note_time(i * 1000);
            note_push_path("fn1");
            note_push_path("fn2");
            note("a", true);
            note_pop_path();
            note("a", rhdl_bits::bits::<6>(i as u128));
            note_pop_path();
        }
        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd).unwrap();
        std::fs::write("test_nested_paths.vcd", vcd).unwrap();
    }
}
