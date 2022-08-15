use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use enumflags2::{bitflags, BitFlags};
use scale_info::{build::Fields, meta_type, prelude::vec, Path, Type, TypeInfo, TypeParameter};
use sp_core::H160;
use sp_runtime::RuntimeDebug;

pub trait UserPrivilegeInterface<Account> {
    fn has_privilege(user: &Account, p: Privilege) -> bool;
    fn has_evm_privilege(user: &H160, p: Privilege) -> bool;
}

impl<Account> UserPrivilegeInterface<Account> for () {
    fn has_privilege(_user: &Account, _p: Privilege) -> bool {
        true
    }

    fn has_evm_privilege(_user: &H160, _p: Privilege) -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Decode, Encode, TypeInfo, RuntimeDebug)]
pub enum PrivilegeMapping {
    LockerMember,
    ReleaseSetter,
    EvmAddressSetter,
    EvmCreditOperation,
    NpowMint,
    CreditAdmin,
    TipPayer,
    BridgeAdmin,
    OracleWorker,
}

impl From<PrivilegeMapping> for Privilege {
    fn from(p: PrivilegeMapping) -> Self {
        match p {
            PrivilegeMapping::LockerMember => Privilege::LockerMember,
            PrivilegeMapping::ReleaseSetter => Privilege::ReleaseSetter,
            PrivilegeMapping::EvmAddressSetter => Privilege::EvmAddressSetter,
            PrivilegeMapping::CreditAdmin => Privilege::CreditAdmin,
            PrivilegeMapping::EvmCreditOperation => Privilege::EvmCreditOperation,
            PrivilegeMapping::NpowMint => Privilege::NpowMint,
            PrivilegeMapping::TipPayer => Privilege::TipPayer,
            PrivilegeMapping::BridgeAdmin => Privilege::BridgeAdmin,
            PrivilegeMapping::OracleWorker => Privilege::OracleWorker,
        }
    }
}

impl From<Privilege> for PrivilegeMapping {
    fn from(p: Privilege) -> Self {
        match p {
            Privilege::LockerMember => PrivilegeMapping::LockerMember,
            Privilege::ReleaseSetter => PrivilegeMapping::ReleaseSetter,
            Privilege::EvmAddressSetter => PrivilegeMapping::EvmAddressSetter,
            Privilege::EvmCreditOperation => PrivilegeMapping::EvmCreditOperation,
            Privilege::NpowMint => PrivilegeMapping::NpowMint,
            Privilege::CreditAdmin => PrivilegeMapping::CreditAdmin,
            Privilege::TipPayer => PrivilegeMapping::TipPayer,
            Privilege::BridgeAdmin => PrivilegeMapping::BridgeAdmin,
            Privilege::OracleWorker => PrivilegeMapping::OracleWorker,
        }
    }
}

#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, PartialEq, Eq, TypeInfo, RuntimeDebug)]
pub enum Privilege {
    LockerMember = 1 << 0,
    ReleaseSetter = 1 << 1,
    EvmAddressSetter = 1 << 2,
    EvmCreditOperation = 1 << 3,
    NpowMint = 1 << 4,
    CreditAdmin = 1 << 5,
    TipPayer = 1 << 6,
    BridgeAdmin = 1 << 7,
    OracleWorker = 1 << 8,
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
