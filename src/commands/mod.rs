pub mod daemon;
pub mod oneshot;

use crate::{operation::Operation, value::ValueWrapper};
use iroha_crypto::prelude::*;
use iroha_data_model::prelude::*;
use iroha_primitives::fixed::Fixed;
use rand::prelude::*;
use std::str::FromStr;

fn make_instruction_by_operation(
    op: &Operation,
    test_account_id: AccountId,
    test_domain_id: DomainId,
    index: usize,
) -> Vec<InstructionBox> {
    match op {
        Operation::RegisterAccount => {
            let new_account_name = Name::from_str(format!("alice{}", index).as_str())
                .expect("Failed to create a new account name");
            let new_account_id: AccountId = AccountId::new(new_account_name, test_domain_id);
            let (public_key, _) = KeyPair::generate()
                .expect("Failed to create a new key pair")
                .into();
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
            let mut new_asset_definition =
                AssetDefinition::quantity(new_asset_definition_id.clone());
            if random() {
                new_asset_definition = new_asset_definition.mintable_once();
            }
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::Quantity(random()),
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
            let mut new_asset_definition =
                AssetDefinition::big_quantity(new_asset_definition_id.clone());
            if random() {
                new_asset_definition = new_asset_definition.mintable_once();
            }
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::BigQuantity(random()),
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
            let mut new_asset_definition = AssetDefinition::fixed(new_asset_definition_id.clone());
            if random() {
                new_asset_definition = new_asset_definition.mintable_once();
            }
            let new_asset = Asset::new(
                AssetId::new(new_asset_definition_id, test_account_id),
                AssetValue::Fixed(Fixed::try_from(random::<f64>()).expect("Valid fixed num")),
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
            let mut new_asset_definition = AssetDefinition::store(new_asset_definition_id.clone());
            if random() {
                new_asset_definition = new_asset_definition.mintable_once();
            }
            let mut store = Metadata::new();
            let val: ValueWrapper = random();
            store
                .insert_with_limits(
                    Name::from_str("Bytes").expect("Failed to create a metadata key"),
                    val.inner(),
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
        Operation::TransferAsset => {
            // Make a new sender asset
            let new_asset_name = Name::from_str(format!("rose{}_to_transfer", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id =
                AssetDefinitionId::new(new_asset_name, test_domain_id.clone());
            let new_asset_definition = AssetDefinition::quantity(new_asset_definition_id.clone());
            let new_sender_asset_id =
                AssetId::new(new_asset_definition_id.clone(), test_account_id);
            let new_sender_asset =
                Asset::new(new_sender_asset_id.clone(), AssetValue::Quantity(1000));

            // Make a new recipient account
            let new_recipient_account_name =
                Name::from_str(format!("bob{}_to_transfer", index).as_str())
                    .expect("Failed to create a new account name");
            let new_recipient_account_id =
                AccountId::new(new_recipient_account_name, test_domain_id);
            let (public_key, _) = KeyPair::generate()
                .expect("Failed to create a new key pair")
                .into();
            let new_recipient_account =
                Account::new(new_recipient_account_id.clone(), [public_key]);

            // Make a new recipient asset
            let new_recipient_asset_id =
                AssetId::new(new_asset_definition_id, new_recipient_account_id);
            let new_recipient_asset =
                Asset::new(new_recipient_asset_id.clone(), AssetValue::Quantity(0));

            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_sender_asset).into(),
                RegisterBox::new(new_recipient_account).into(),
                RegisterBox::new(new_recipient_asset).into(),
                TransferBox::new(
                    IdBox::AssetId(new_sender_asset_id),
                    1_u32,
                    IdBox::AssetId(new_recipient_asset_id),
                )
                .into(),
            ]
        }
        Operation::MintAsset => {
            // Make a new asset
            let new_asset_name = Name::from_str(format!("rose{}_to_mint", index).as_str())
                .expect("Failed to create a new asset name");
            let new_asset_definition_id =
                AssetDefinitionId::new(new_asset_name, test_domain_id.clone());
            let new_asset_definition = AssetDefinition::quantity(new_asset_definition_id.clone());
            let new_asset_id = AssetId::new(new_asset_definition_id, test_account_id);
            let new_asset = Asset::new(new_asset_id.clone(), AssetValue::Quantity(0));

            // Make a new account
            let new_account_name = Name::from_str(format!("bob{}_to_mint", index).as_str())
                .expect("Failed to create a new account name");
            let new_account_id = AccountId::new(new_account_name, test_domain_id);
            let (public_key, _) = KeyPair::generate()
                .expect("Failed to create a new key pair")
                .into();
            let new_account = Account::new(new_account_id, [public_key]);

            vec![
                RegisterBox::new(new_asset_definition).into(),
                RegisterBox::new(new_asset).into(),
                RegisterBox::new(new_account).into(),
                MintBox::new(1_u32, IdBox::AssetId(new_asset_id)).into(),
            ]
        }
    }
}
