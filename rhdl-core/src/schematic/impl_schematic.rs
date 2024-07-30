use crate::rtl::{object::RegisterKind, Object};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentIx(usize);

impl std::fmt::Debug for ComponentIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

const ORPHAN: ComponentIx = ComponentIx(!0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinIx(usize);

impl std::fmt::Debug for PinIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub kind: RegisterKind,
    pub name: String,
    pub parent: ComponentIx,
}

impl Pin {
    pub fn parent(&mut self, parent: ComponentIx) {
        self.parent = parent;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Wire {
    pub source: PinIx,
    pub dest: PinIx,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub inputs: Vec<PinIx>,
    pub outputs: Vec<PinIx>,
}

#[derive(Debug, Clone)]
pub struct BlackBoxComponent {}

#[derive(Debug, Clone)]
pub enum ComponentKind {
    RTL(Object),
    BlackBox(BlackBoxComponent),
    Buffer,
    Concat,
    Split,
    Schematic(Box<Schematic>),
}

#[derive(Clone, Debug)]
pub struct Schematic {
    pub pins: Vec<Pin>,
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    pub inputs: Vec<PinIx>,
    pub output: PinIx,
}

impl Schematic {
    pub fn make_pin(&mut self, kind: RegisterKind, name: String) -> PinIx {
        let pin = Pin {
            kind,
            name,
            parent: ORPHAN,
        };
        self.pins.push(pin);
        PinIx(self.pins.len() - 1)
    }
    pub fn make_buffer(&mut self, kind: RegisterKind) -> (PinIx, PinIx) {
        let input = self.make_pin(kind, "in".to_string());
        let output = self.make_pin(kind, "out".to_string());
        let buf = self.make_component(ComponentKind::Buffer, vec![input], vec![output]);
        self.pin_mut(input).parent(buf);
        self.pin_mut(output).parent(buf);
        (input, output)
    }
    pub fn make_component(
        &mut self,
        kind: ComponentKind,
        inputs: Vec<PinIx>,
        outputs: Vec<PinIx>,
    ) -> ComponentIx {
        let component = Component {
            kind,
            inputs,
            outputs,
        };
        self.components.push(component);
        ComponentIx(self.components.len() - 1)
    }
    pub fn pin(&self, ix: PinIx) -> &Pin {
        &self.pins[ix.0]
    }
    pub fn pin_mut(&mut self, ix: PinIx) -> &mut Pin {
        &mut self.pins[ix.0]
    }
    pub fn wire(&mut self, source: PinIx, dest: PinIx) {
        self.wires.push(Wire { source, dest });
    }
}
