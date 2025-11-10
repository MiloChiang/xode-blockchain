use crate::Balance;
use xcm::latest::prelude::*;
use xcm_executor::traits::{Error as MatchError, MatchesFungible, MatchesFungibles};

/// Asset matcher for handling various fungible assets in XCM transactions.
/// 
/// This struct implements the `MatchesFungible` and `MatchesFungibles` traits
/// to match and extract balances for different types of assets, including the
/// native token of the parachain, assets from the relay chain, and assets from
/// sibling parachains (e.g., AssetHub).
pub struct AssetMatcher;

/// Implementation of `MatchesFungible` for matching the native token.
/// 
/// This implementation checks if the asset is the native token of the parachain
/// by verifying if its location is `Here` with zero parents. If matched, it
/// returns the balance amount.
impl MatchesFungible<Balance> for AssetMatcher {
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

/// Implementation of `MatchesFungibles` for matching various fungible assets.
/// 
/// This implementation checks for assets from the local parachain, relay chain,
/// and sibling parachains (e.g., AssetHub). It returns the corresponding asset ID
/// and balance if matched.
impl MatchesFungibles<u32, Balance> for AssetMatcher {
    fn matches_fungibles(asset: &Asset) -> Result<(u32, Balance), MatchError> {
        let match_result = match asset {
            // Match a local parachain (e.g., Xode)
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
