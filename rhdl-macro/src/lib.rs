use proc_macro::TokenStream;

#[proc_macro_derive(Digital, attributes(rhdl))]
pub fn digital(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_digital(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Notable, attributes(rhdl))]
pub fn notable(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_notable(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Timed)]
pub fn timed(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_timed(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn kernel(_attr: TokenStream, input: TokenStream) -> TokenStream {
    match rhdl_macro_core::hdl_kernel(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn hdl(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::hdl_kernel(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Circuit, attributes(rhdl))]
pub fn circuit(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_circuit(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(CircuitDQZ)]
pub fn circuit_dqz(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_circuit_dqz(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Synchronous, attributes(rhdl))]
pub fn synchronous(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_synchronous(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SynchronousDQZ)]
pub fn synchronous_dqz(input: TokenStream) -> TokenStream {
    match rhdl_macro_core::derive_synchronous_dqz(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
