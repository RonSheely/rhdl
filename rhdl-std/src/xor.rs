use rhdl_bits::Bits;
use rhdl_core::digital_fn::DigitalFn;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;

pub fn xor<const N: usize>(x: Bits<N>) -> bool {
    let mut x = x.0;
    x ^= x >> 1;
    x ^= x >> 2;
    x ^= x >> 4;
    x ^= x >> 8;
    x ^= x >> 16;
    x ^= x >> 32;
    x & 1 == 1
}

#[allow(non_camel_case_types)]
pub struct xor<const N: usize> {}

impl<const N: usize> DigitalFn for xor<N> {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: format!("xor_{N}"),
            body: format!(
                "function [{}:0] xor_{N}(input [{}:0] a); xor_{N} = ^a; endfunction",
                N - 1,
                N - 1
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor() {
        let bits = Bits::<128>::mask();
        assert!(!xor(bits));
        let bits = Bits::<32>::mask();
        assert!(!xor(bits));
        let bits = Bits::<1>::mask();
        assert!(xor(bits));
        let bits: Bits<5> = 0b11010.into();
        assert!(xor(bits));
    }
}
