use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use enumflags2::{bitflags, BitFlags};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter,prelude::vec};
use sp_core::H160;

pub trait UserPrivilegeInterface<Account> {
    fn has_privilege(user: &Account, p: Privilege) -> bool;
    fn has_evm_privilege(user: &H160, p: Privilege) -> bool;
}

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Privilege {
    LockerMember = 1 << 0,
    ReleaseSetter = 1 << 1,
    EvmAddressSetter = 1 << 2,
    EvmCreditOperation = 1 << 3,
    NpowMint = 1 << 4,
    CreditAdmin = 1 << 5,
    TipPayer = 1 << 6,
}

/// Wrapper type for `BitFlags<Privilege>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Privileges(pub BitFlags<Privilege>);

impl MaxEncodedLen for Privileges {
    fn max_encoded_len() -> usize {
        u64::max_encoded_len()
    }
}

impl Eq for Privileges {}
impl Encode for Privileges {
    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        self.0.bits().using_encoded(f)
    }
}

impl EncodeLike for Privileges {}

impl Decode for Privileges {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let field = u64::decode(input)?;
        Ok(Self(
            <BitFlags<Privilege>>::from_bits(field as u64).map_err(|_| "invalid value")?,
        ))
    }
}
impl TypeInfo for Privileges {
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("BitFlags", module_path!()))
            .type_params(vec![TypeParameter::new(
                "T",
                Some(meta_type::<Privilege>()),
            )])
            .composite(Fields::unnamed().field(|f| f.ty::<u64>().type_name("Privilege")))
    }
}