pub mod daemon;
pub mod oneshot;

use crate::operation::Operation;
use iroha_crypto::prelude::*;
use iroha_data_model::prelude::*;
use iroha_primitives::fixed::Fixed;
use std::str::FromStr;

fn make_instruction_by_operation(
    op: &Operation,
    test_account_id: AccountId,
    test_domain_id: DomainId,
    index: usize,
) -> Vec<Instruction> {
    match op {
        Operation::RegisterAccount => {
            let new_account_name = Name::from_str(format!("alice{}", index).as_str())
                .expect("Failed to create a new account name");
            let new_account_id: AccountId = AccountId::new(new_account_name, test_domain_id);
            let (public_key, _) = KeyPair::generate().unwrap().into();
            vec![RegisterBox::new(Account::new(new_account_id, [public_key])).into()]
        }
        Operation::RegisterDomain => {
            let new_domain_name = Name::from_str(format!("wonderland{}", index).as_str())
                .expect("Failed to create a new domain name");
            let new_domain_id: DomainId = DomainId::new(new_domain_name);
            vec![RegisterBox::new(Domain::new(new_domain_id)).into()]
        }
        Operation::RegisterAssetQuantity => {
            let new_asset_name = Name::from_str(format!("rose_quantity{}", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id: AssetDefinitionId =
                AssetDefinitionId::new(new_asset_name, test_domain_id);
            let new_asset_definition = AssetDefinition::quantity(new_asset_definition_id.clone());
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::Quantity(1000),
            );
            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_asset).into(),
            ]
        }
        Operation::RegisterAssetBigQuantity => {
            let new_asset_name = Name::from_str(format!("rose_big_quantity{}", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id: AssetDefinitionId =
                AssetDefinitionId::new(new_asset_name, test_domain_id);
            let new_asset_definition =
                AssetDefinition::big_quantity(new_asset_definition_id.clone());
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::BigQuantity(100000000999900u128),
            );
            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_asset).into(),
            ]
        }
        Operation::RegisterAssetFixed => {
            let new_asset_name = Name::from_str(format!("rose_fixed{}", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id: AssetDefinitionId =
                AssetDefinitionId::new(new_asset_name, test_domain_id);
            let new_asset_definition = AssetDefinition::fixed(new_asset_definition_id.clone());
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::Fixed(Fixed::try_from(1000f64).expect("Valid fixed num")),
            );
            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_asset).into(),
            ]
        }
        Operation::RegisterAssetStore => {
            let new_asset_name = Name::from_str(format!("rose_store{}", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id: AssetDefinitionId =
                AssetDefinitionId::new(new_asset_name, test_domain_id);
            let new_asset_definition = AssetDefinition::store(new_asset_definition_id.clone());
            let mut store = Metadata::new();
            store
                .insert_with_limits(
                    Name::from_str("Bytes").expect("Failed to create a metadata key"),
                    Value::Vec(vec![Value::U32(99), Value::U32(98), Value::U32(300)]),
                    MetadataLimits::new(10, 100),
                )
                .expect("Insert some metadata");
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::Store(store),
            );
            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_asset).into(),
            ]
        }
    }
}
