use crate::{
    digital_fn::DigitalFn2, error::RHDLError, flow_graph::optimization::optimize_flow_graph,
    Digital, DigitalFn, FlowGraph, Timed,
};

use super::{circuit_descriptor::CircuitDescriptor, hdl_descriptor::HDLDescriptor};

pub trait CircuitIO: 'static + Sized + Clone + CircuitDQ {
    type I: Timed;
    type O: Timed;
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
}

pub trait CircuitDQ: 'static + Sized + Clone {
    type D: Timed;
    type Q: Timed;
}

pub trait Circuit: 'static + Sized + Clone + CircuitIO {
    // State for simulation - auto derived
    type S: Digital;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    // auto derived
    fn description(&self) -> String {
        format!("circuit {}", std::any::type_name::<Self>())
    }

    // auto derived
    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError>;

    // auto derived
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError>;

    // Return a top level flow graph for this circuit, optimized and sealed.
    fn flow_graph(&self, name: &str) -> Result<FlowGraph, RHDLError> {
        let flow_graph = self.descriptor(name)?.flow_graph.clone();
        optimize_flow_graph(flow_graph)
    }
}
