pub(crate) mod check_clock_coherence;
pub(crate) mod check_for_rolled_types;
pub(crate) mod check_rhif_flow;
pub(crate) mod check_rhif_type;
pub(crate) mod dead_code_elimination;
pub(crate) mod lower_dynamic_indices_with_constant_arguments;
pub(crate) mod lower_index_to_copy;
pub(crate) mod lower_inferred_casts;
pub(crate) mod lower_inferred_retimes;
pub(crate) mod pass;
pub(crate) mod pre_cast_literals;
pub(crate) mod precast_integer_literals_in_binops;
pub(crate) mod precompute_discriminants;
pub(crate) mod remove_empty_cases;
pub(crate) mod remove_extra_registers;
pub(crate) mod remove_unneeded_muxes;
pub(crate) mod remove_unused_literals;
pub(crate) mod remove_unused_registers;
pub(crate) mod remove_useless_casts;
pub(crate) mod symbol_table_is_complete;
