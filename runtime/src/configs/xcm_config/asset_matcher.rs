use crate::Balance;
use xcm::latest::prelude::*;
use xcm_executor::traits::{Error as MatchError, MatchesFungible, MatchesFungibles};

/// This struct defines a native asset matcher for XCM transactions.
/// 
/// It identifies the native token based on its XCM location and fungibility,
/// determining if it matches the native asset.
pub struct NativeAssetMatcher;

impl MatchesFungible<Balance> for NativeAssetMatcher {
    fn matches_fungible(asset: &Asset) -> Option<Balance> {
        match asset {
            // Match the native token
            Asset {
                id: AssetId(Location {
                    parents: 0,
                    interior: Junctions::Here,
                }),
                fun: Fungibility::Fungible(amount),
            } => {
                log::trace!(target: "xcm::matches_fungible", "AssetMatcher: Matched native token, amount: {:?}", amount);
                Some(*amount)
            },

            // Otherwise, mismatched asset type
            _ => {
                log::trace!(target: "xcm::matches_fungible", "AssetMatcher: Asset not handled → asset: {:?}", asset);
                None
            }
        }
    }
}

/// This struct defines a multi-asset matcher for XCM transactions.
/// 
/// It identifies various fungible assets based on their XCM location and fungibility,
/// determining if they match known assets and extracting their identifiers and amounts.
pub struct MultiAssetMatcher;

impl MatchesFungibles<u32, Balance> for MultiAssetMatcher {
    fn matches_fungibles(asset: &Asset) -> Result<(u32, Balance), MatchError> {
        let match_result = match asset {

            // Match a local parachain (Xode)
            Asset {
                id: AssetId(Location {
                    parents: 0,
                    interior: Junctions::X2(junctions),
                }),
                fun: Fungibility::Fungible(amount),
            } => match junctions.as_ref() {
                [Junction::PalletInstance(50), Junction::GeneralIndex(asset_id)] => {
                    log::trace!(target: "xcm::matches_fungibles", "AssetMatcher: Matched Xode asset → asset_id: {:?}, amount: {:?}", asset_id, amount);
                    Ok((*asset_id as u32, *amount))
                }
                _ => Err(MatchError::AssetNotHandled),
            },

            // Match the relay chain (parent)
            Asset {
                id: AssetId(Location {
                    parents: 1,
                    interior: Junctions::Here,
                }),
                fun: Fungibility::Fungible(amount),
            } => {
                log::trace!(target: "xcm::matches_fungibles", "AssetMatcher: Matched Relay Chain native asset: amount = {:?}", amount);
                Ok((100_000_000, *amount))
            }

            // Match a sibling parachain (e.g., AssetHub with ParaId 1000)
            Asset {
                id: AssetId(Location {
                    parents: 1,
                    interior: Junctions::X3(junctions),
                }),
                fun: Fungibility::Fungible(amount),
            } => match junctions.as_ref() {
                [Junction::Parachain(1000), Junction::PalletInstance(50), Junction::GeneralIndex(asset_id)] =>
                {
                    if *asset_id == 100_000_000 {
                        log::warn!(target: "xcm::matches_fungibles", "AssetMatcher: Blocked AssetHub asset with DOT ID collision");
                        return Err(MatchError::AssetNotHandled);
                    }

                    log::trace!(target: "xcm::matches_fungibles", "AssetMatcher: Matched AssetHub asset → asset_id: {:?}, amount: {:?}", asset_id, amount);
                    Ok((*asset_id as u32, *amount))
                }
                _ => Err(MatchError::AssetNotHandled),
            },

            // Otherwise, mismatched asset type
            _ => {
                log::trace!(target: "xcm::matches_fungibles", "AssetMatcher: Asset not handled → asset: {:?}", asset);
                Err(MatchError::AssetNotHandled)
            }
        };

        log::trace!(target: "xcm::matches_fungibles", "AssetMatcher: Final result for asset {:?} → {:?}", asset, match_result);
        match_result
    }
}
