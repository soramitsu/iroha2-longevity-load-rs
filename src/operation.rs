use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString)]
#[allow(clippy::enum_variant_names)]
pub enum Operation {
    RegisterAccount,
    RegisterDomain,
    RegisterAssetQuantity,
    RegisterAssetBigQuantity,
    RegisterAssetFixed,
    RegisterAssetStore,
    TransferAsset,
    MintAsset,
}
