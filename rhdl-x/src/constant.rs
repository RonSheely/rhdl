use rhdl::prelude::*;

#[derive(Clone, Debug)]
pub struct U<T: Digital> {
    value: T,
}

impl<T: Digital> U<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = ();
    type O = T;
}

impl<T: Digital> SynchronousDQ for U<T> {
    type D = ();
    type Q = ();
}

impl<T: Digital> Synchronous for U<T> {
    type Update = Self;

    type S = ();

    type Z = ();

    fn sim(
        &self,
        _clock_reset: ClockReset,
        _input: Self::I,
        _state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        self.value
    }

    fn name(&self) -> String {
        "Constant".into()
    }

    fn as_hdl(&self, _kind: HDLKind) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog()
    }

    fn descriptor(&self) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let my_val = &self.value.typed_bits().bits;
        let driver = my_val.iter().map(|b| {
            flow_graph.new_component_with_optional_location(ComponentKind::Constant(*b), 1, None)
        });
        flow_graph.output = driver.collect();
        flow_graph.inputs = vec![vec![], vec![]];
        Ok(CircuitDescriptor {
            unique_name: format!("const_{:?}", self.value.typed_bits()),
            input_kind: Kind::Empty,
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            num_tristate: 0,
            tristate_offset_in_parent: 0,
            flow_graph,
            children: Default::default(),
            rtl: None,
        })
    }
}

impl<T: Digital> DigitalFn for U<T> {}

impl<T: Digital> U<T> {
    fn as_verilog(&self) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor()?.unique_name;
        let output_bits = T::bits().saturating_sub(1);
        let value = self.value.typed_bits().as_verilog_literal();
        let body = format!(
            "
module {module_name}(input clock, input reset, output wire[{output_bits}:0] o);
    assign o = {value};
endmodule
"
        );
        Ok(HDLDescriptor {
            name: module_name,
            body,
            children: Default::default(),
        })
    }
}
