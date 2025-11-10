use crate::{WeightToFee, configs::xcm_config::AssetTransactor};
use frame_support::{
    parameter_types,
    weights::{Weight, WeightToFee as WeightToFeeT},
};
use xcm::latest::prelude::*;
use xcm_executor::{
	AssetsInHolding,
	traits::{TransactAsset, WeightTrader}
};
use alloc::sync::Arc;
use sp_core::crypto::{Ss58Codec, AccountId32};

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
}

/// A weight to fee implementation for USDT, which is used to convert weight into a fee
/// that can be paid in USDT. This implementation is specifically designed to handle
/// the conversion of weight into a fee amount that can be used for weight purchasing
/// in the context of XCM transactions.
/// 
/// The fee is calculated based on the weight's reference time, divided by a scaling factor
/// to convert it into a fee amount in USDT.
/// 
/// The scaling factor is set to 1,000,000 to ensure that the fee is reasonable and
/// can be handled by the USDT asset.
/// 
/// This implementation is useful for scenarios where USDT is used as the asset for weight purchasing,
/// allowing for dynamic handling of weight purchasing based on the available assets in the `AssetsInHolding`.
pub struct UsdtWeightToFee;

impl WeightToFeeT for UsdtWeightToFee {
	type Balance = u128;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		weight.ref_time().saturating_div(1_000_000).max(1).into()
	}
}

/// A dynamic weight trader that determines how to buy weight for XCM execution  
/// based on the assets provided in the `AssetsInHolding`. This trader supports  
/// both DOT from the Relay Chain and USDT from AssetHub (parachain 1000).  
///
/// It matches assets by their XCM location and uses the corresponding weight-to-fee  
/// conversion logic to calculate the cost for the requested weight.  
///
/// If the asset originates from the Relay Chain, the trader uses DOT for payment.  
/// If it originates from AssetHub, it uses USDT. The function processes the payment,  
/// deducts the total fee, and returns any remaining balance to the caller.  
///
/// If no supported asset is detected, the trader returns an error indicating that  
/// the weight purchase cannot be completed.
pub struct DynamicWeightTrader;

impl WeightTrader for DynamicWeightTrader {
	fn new() -> Self {
		Self
	}

	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: AssetsInHolding,
		context: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		log::trace!(target: "xcm::weight_trader", "DynamicWeightTrader::buy_weight - weight: {:?}, payment: {:?}, context: {:?}",  weight, payment, context);

		// Early return if payment is empty
		let (asset_id, _) = payment.fungible.iter().next().ok_or(XcmError::TooExpensive)?;

		// Check if asset matches USDT on AssetHub (parachain 1000)
		match asset_id {
			AssetId(dot_location @ Location {
				parents: 1,
				interior: Junctions::Here,
			}) => handle_payment(dot_location.clone(), payment, WeightToFee::weight_to_fee(&weight)),

			AssetId(usdt_location @ Location {
				parents: 1,
				interior: Junctions::X3(junctions),
			}) if matches!(
				junctions.as_ref(),
				[
					Junction::Parachain(1000),
					Junction::PalletInstance(50),
					Junction::GeneralIndex(1984)
				]
			) => handle_payment(usdt_location.clone(), payment, UsdtWeightToFee::weight_to_fee(&weight)),
			
			_ => {
				log::trace!(target: "xcm::weight_trader", "DynamicWeightTrader::buy_weight - Unsupported asset: {:?}", asset_id);
				Err(XcmError::AssetNotFound)
			}
		}
	}

	fn refund_weight(&mut self, _weight: Weight, _context: &XcmContext) -> Option<Asset> {
		None
	}
}

/// Handles a payment for purchasing XCM execution weight, supporting both DOT and USDT.  
/// The function validates the provided asset balance and ensures it can fully cover the total cost.  
/// The total fee consists of the required XCM execution fee plus a fixed surcharge of 0.01 (DOT or USDT).  
///
/// If the asset originates from the Relay Chain, the payment is processed using DOT.  
/// If the asset comes from AssetHub (parachain 1000), it is processed using USDT.  
/// The combined fee amount is then transferred to a predefined receiver account,  
/// which is converted into an XCM-compatible location for proper asset deposit handling.  
///
/// After deducting the total fee (execution fee + 0.01), the function returns any remaining  
/// balance to the caller. Detailed logging throughout the process ensures transparency  
/// and simplifies debugging and auditing.
fn handle_payment(
	asset_location: Location,
	payment: AssetsInHolding,
	fee_amount: u128,
) -> Result<AssetsInHolding, XcmError> {
	let fixed_fee: u128 = match asset_location.clone() {
		// DOT: relay chain (10 decimals -> 0.01 DOT = 100_000_000)
		Location {
			parents: 1,
			interior: Junctions::Here,
		} => 100_000_000,

		// USDT: AssetHub (6 decimals -> 0.01 USDT = 10_000)
		Location {
			parents: 1,
			interior: Junctions::X3(junctions),
		} if matches!(
			junctions.as_ref(),
			[
				Junction::Parachain(1000),
				Junction::PalletInstance(50),
				Junction::GeneralIndex(1984)
			]
		) => 10_000,

		_ => {
			log::error!(
				target: "xcm::weight_trader", "handle_payment - Unsupported asset location: {:?}",
				asset_location
			);
			return Err(XcmError::FailedToTransactAsset("Unsupported asset location"));
		}
	};

	let total_fee = fee_amount.saturating_add(fixed_fee);

	// Subtract total fee from payment
	let required_asset: Asset = (AssetId(asset_location.clone()), total_fee).into();
	let unused_assets = payment.checked_sub(required_asset).map_err(|_| XcmError::TooExpensive)?;

	// Remaining balance
	let (_, remaining_balance) = unused_assets.fungible.iter().next().unwrap_or((&AssetId(asset_location.clone()), &0));

	// Receiver (fee collector)
	let receiver = AccountId32::from_ss58check("5Fqc6tG16FcR4QLwknqs5sb9Pc6pqsNwtwcPh7fNKCaP4DJT")
		.map_err(|e| {
			log::error!(target: "xcm::weight_trader", "Invalid receiver address: {:?}", e);
			XcmError::FailedToTransactAsset("Invalid receiver address")
		})?;

	let receiver_location = Location {
		parents: 0,
		interior: Junctions::X1(Arc::from([Junction::AccountId32 {
			network: None,
			id: receiver.into(),
		}])),
	};

	// Deposit total fee
	let fee_asset: Asset = (AssetId(asset_location.clone()), total_fee).into();
	AssetTransactor::deposit_asset(&fee_asset, &receiver_location, None).map_err(|e| {
		log::error!(target: "xcm::weight_trader", "Fee deposit failed for {:?}: {:?}", asset_location, e);
		XcmError::FailedToTransactAsset("Fee deposit failed")
	})?;

	// Return remaining assets (if any)
	let mut final_assets = AssetsInHolding::new();
	if *remaining_balance > 0 {
		final_assets.subsume((AssetId(asset_location.clone()), *remaining_balance).into());
	}

	log::trace!(
		target: "xcm::weight_trader", "handle_payment - Processed {:?}: total_fee={:?} (+0.01), remaining={:?}",
		asset_location, total_fee, remaining_balance
	);

	Ok(final_assets)
}