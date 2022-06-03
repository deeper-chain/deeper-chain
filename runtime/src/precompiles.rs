use pallet_evm::{Context, Precompile, PrecompileResult, PrecompileSet};
use sp_core::H160;
use sp_std::marker::PhantomData;

use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_curve25519::{Curve25519Add, Curve25519ScalarMul};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_ed25519::Ed25519Verify;

pub struct FrontierPrecompiles<R>(PhantomData<R>);

impl<R> FrontierPrecompiles<R>
where
    R: pallet_evm::Config,
{
    pub fn new() -> Self {
        Self(Default::default())
    }
    pub fn used_addresses() -> sp_std::vec::Vec<H160> {
        sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027, 1028, 1029]
            .into_iter()
            .map(hash)
            .collect()
    }
}
impl<R> PrecompileSet for FrontierPrecompiles<R>
where
    Dispatch<R>: Precompile,
    R: pallet_evm::Config,
{
    fn execute(
        &self,
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> Option<PrecompileResult> {
        match address {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
            a if a == hash(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
            a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
            a if a == hash(4) => Some(Identity::execute(input, target_gas, context, is_static)),
            a if a == hash(5) => Some(Modexp::execute(input, target_gas, context, is_static)),
            a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context, is_static)),
            a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context, is_static)),
            a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context, is_static)),
            a if a == hash(9) => Some(Blake2F::execute(input, target_gas, context, is_static)),
            // Non-Frontier specific nor Ethereum precompiles :
            a if a == hash(1024) => {
                Some(Sha3FIPS256::execute(input, target_gas, context, is_static))
            }
            a if a == hash(1025) => Some(ECRecoverPublicKey::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1026) => Some(Dispatch::<R>::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1027) => Some(Curve25519Add::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1028) => Some(Curve25519ScalarMul::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1029) => Some(Ed25519Verify::execute(
                input, target_gas, context, is_static,
            )),
            _ => None,
        }
    }

    fn is_precompile(&self, address: H160) -> bool {
        Self::used_addresses().contains(&address)
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}
