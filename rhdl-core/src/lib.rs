pub use types::kind::Kind;
pub mod clock_details;

pub use circuit::circuit_descriptor::root_descriptor;
pub use circuit::circuit_descriptor::CircuitDescriptor;
pub use circuit::circuit_impl::Circuit;
pub use circuit::circuit_impl::CircuitIO;
pub use circuit::circuit_impl::HDLKind;
pub use circuit::hdl_descriptor::root_hdl;
pub use circuit::hdl_descriptor::root_synchronous_hdl;
pub use circuit::hdl_descriptor::HDLDescriptor;
pub use circuit::synchronous::synchronous_root_descriptor;
pub use circuit::synchronous::Synchronous;
pub use circuit::synchronous::SynchronousDQ;
pub use circuit::synchronous::SynchronousIO;
pub use circuit::verilog::root_verilog;
pub use clock_details::ClockDetails;
pub use crusty::check_schematic;
pub use types::bitz::BitZ;
pub use types::clock::Clock;
pub use types::digital::Digital;
pub use types::digital_fn::DigitalFn;
pub use types::domain::Color;
pub use types::domain::Domain;
pub use types::kernel::KernelFnKind;
#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid;
#[cfg(feature = "svg")]
pub use types::kind::kind_svg::svg_grid_vertical;
pub use types::kind::text_grid;
pub use types::kind::DiscriminantAlignment;
pub use types::kind::VariantType;
pub use types::note::Notable;
pub use types::register::Register;
pub use types::register::SignedRegister;
pub use types::reset::Reset;
pub use types::signal::Signal;
pub use types::timed::Timed;
pub use types::tristate::Tristate;
pub mod ast;
pub mod circuit;
pub mod codegen;
pub mod compiler;
pub mod crusty;
//pub mod diagnostic;
pub mod dyn_bit_manip;
pub mod note_db;
pub mod schematic;
pub mod test_module;
pub mod types;
pub mod util;
pub use util::id;

pub use codegen::verilog::as_verilog_literal;
pub use codegen::verilog::generate_verilog;
pub use codegen::verilog::VerilogModule;
pub use compiler::compile_design;
pub use note_db::note;
pub use note_db::note_init_db;
pub use note_db::note_pop_path;
pub use note_db::note_push_path;
pub use note_db::note_take;
pub use note_db::note_time;
pub use note_db::NoteDB;
pub use schematic::components::BlackBoxComponent;
pub use schematic::components::BlackBoxTrait;
pub use schematic::constraints::constraint_input_synchronous;
pub use schematic::constraints::constraint_must_clock;
pub use schematic::constraints::constraint_not_constant_valued;
pub use schematic::constraints::constraint_output_synchronous;
pub use schematic::constraints::Constraint;
pub use schematic::constraints::EdgeType;
pub use test_module::test_kernel_vm_and_verilog;
#[cfg(feature = "iverilog")]
pub use test_module::test_with_iverilog;
pub use types::kind::DiscriminantType;
pub use types::note::NoteKey;
pub use types::note::NoteWriter;
pub use types::typed_bits::TypedBits;
pub mod rhif;
pub use ast::ast_builder;
pub use rhif::module::Module;
pub use types::clock;
pub use types::digital_fn;
pub use types::digital_fn::DigitalSignature;
pub use types::kernel;

pub const MAX_ITERS: usize = 10;
pub mod error;
pub use error::RHDLError;
