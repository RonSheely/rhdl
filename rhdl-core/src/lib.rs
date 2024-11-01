pub use types::kind::Kind;
pub mod clock_details;

pub use circuit::circuit_descriptor::CircuitDescriptor;
pub use circuit::circuit_impl::Circuit;
pub use circuit::circuit_impl::CircuitDQZ;
pub use circuit::circuit_impl::CircuitIO;
pub use circuit::hdl_descriptor::HDLDescriptor;
pub use circuit::synchronous::Synchronous;
pub use circuit::synchronous::SynchronousDQZ;
pub use circuit::synchronous::SynchronousIO;
pub use clock_details::ClockDetails;
pub use types::bitz::BitZ;
pub use types::clock::Clock;
pub use types::digital::Digital;
pub use types::digital_fn::DigitalFn;
pub use types::digital_fn::DigitalFn2;
pub use types::digital_fn::DigitalFn3;
pub use types::domain::Color;
pub use types::domain::Domain;
pub use types::kernel::KernelFnKind;
#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid;
#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid_vertical;
pub use types::kind::text_grid;
pub use types::kind::DiscriminantAlignment;
pub use types::register::Register;
pub use types::register::SignedRegister;
pub use types::reset::Reset;
pub use types::signal::Signal;
pub use types::timed::Timed;
pub use types::tristate::Tristate;
pub mod ast;
pub mod circuit;
pub mod compiler;
pub mod dyn_bit_manip;
pub mod types;
pub mod util;
pub use util::id;
pub mod testbench;

pub use compiler::compile_design;
pub use trace::db::trace;
pub use trace::db::trace_init_db;
pub use trace::db::trace_pop_path;
pub use trace::db::trace_push_path;
pub use trace::db::trace_time;
pub use trace::db::TraceDB;
pub use trace::key::TraceKey;
pub use types::kind::DiscriminantType;
pub use types::typed_bits::TypedBits;
pub mod rhif;
pub use ast::ast_builder;
pub use types::clock;
pub use types::digital_fn;
pub use types::digital_fn::DigitalSignature;
pub use types::kernel;

pub const MAX_ITERS: usize = 10;
pub mod error;
pub use error::RHDLError;
pub mod flow_graph;
pub mod rtl;
pub mod timing;
pub use circuit::circuit_descriptor::build_descriptor;
pub use circuit::circuit_descriptor::build_synchronous_descriptor;
pub use circuit::hdl_backend::build_hdl;
pub use circuit::hdl_backend::build_synchronous_hdl;
pub use compiler::CompilationMode;
pub use flow_graph::build_rtl_flow_graph;
pub use flow_graph::flow_graph_impl::FlowGraph;
pub use types::clock_reset::clock_reset;
pub use types::clock_reset::ClockReset;

pub mod sim;
pub use types::timed_sample::timed_sample;
pub use types::timed_sample::TimedSample;
pub mod hdl;
pub use sim::waveform::waveform_synchronous;
pub mod trace;
pub use dyn_bit_manip::move_nbits_to_msb;
