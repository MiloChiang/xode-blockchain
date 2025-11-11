use frame_support::traits::ContainsPair;
use xcm::latest::prelude::*;

/// This struct defines trusted reserve assets for XCM transactions.
///
/// It identifies assets based on their XCM location and the origin of the XCM message,
/// determining if they are considered trusted reserve assets.
pub struct TrustedReserveAssets;

impl ContainsPair<Asset, Location> for TrustedReserveAssets {
    fn contains(asset: &Asset, origin: &Location) -> bool {
        log::trace!(target: "xcm::contains_pair", "TrustedReserveAssets::contains - asset: {:?}, origin: {:?}", asset, origin);

        let result = match &origin {

            // Match the relay chain (parent) origin.
            Location {
                parents: 1,
                interior: Junctions::Here,
            } if matches!(
                &asset.id,
                AssetId(Location {
                    parents: 1,
                    interior: Junctions::Here
                })
            ) => true,

            // Match a sibling parachain origin (e.g., AssetHub with ParaId 1000).
            Location {
                parents: 1,
                interior: Junctions::X1(parachain_junction),
            } => match parachain_junction.as_ref() {
                
                // Match AssetHub (parachain 1000) origin.
                [Junction::Parachain(1000)] => match &asset.id {

                    // Outbound: XCM assets (pallet-assets) from Xode to AssetHub.
                    AssetId(Location {
                        parents: 0,
                        interior: Junctions::X2(asset_junctions),
                    }) if matches!(
                        asset_junctions.as_ref(),
                        [Junction::PalletInstance(50), Junction::GeneralIndex(_)]
                    ) => true,

                    // Inbound: XCM assets (pallet-assets) from AssetHub to Xode.
                    AssetId(Location {
                        parents: 1,
                        interior: Junctions::X3(asset_junctions),
                    }) if matches!(
                        asset_junctions.as_ref(),
                        [
                            Junction::Parachain(1000),
                            Junction::PalletInstance(50),
                            Junction::GeneralIndex(_)
                        ]
                    ) => true,

                    _ => false
                },

                // Match Hydration (parachain 2034) origin.
                [Junction::Parachain(2034)] => match &asset.id {

                    // Outbound: XCM assets (native XON) from Xode to Hydration.
                    AssetId(Location {
                        parents: 0,
                        interior: Junctions::Here,
                    }) => true,

                    // Outbound: XCM assets (pallet-assets) from Xode to Hydration.
                    AssetId(Location {
                        parents: 0,
                        interior: Junctions::X2(asset_junctions),
                    }) if matches!(
                        asset_junctions.as_ref(),
                        [Junction::PalletInstance(50), Junction::GeneralIndex(_)]
                    ) => true,

                    // Inbound: XCM assets (pallet-assets) from Hydration to Xode.
                    AssetId(Location {
                        parents: 1,
                        interior: Junctions::X3(asset_junctions),
                    }) if matches!(
                        asset_junctions.as_ref(),
                        [
                            Junction::Parachain(2034),
                            Junction::PalletInstance(51),
                            Junction::GeneralIndex(_)
                        ]
                    ) => true,

                    _ => false
                },

                _ => false
            },

            _ => false
        };

        log::trace!(target: "xcm::contains_pair", "TrustedReserveAssets::contains - asset: {:?}, origin: {:?} → result: {:?}", asset, origin, result);
        result
    }
}
