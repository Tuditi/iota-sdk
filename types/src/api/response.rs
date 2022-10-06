// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    api::{
        dto::{LedgerInclusionStateDto, PeerDto, ReceiptDto},
        error::Error,
    },
    block::{
        output::{dto::OutputDto, RentStructure, RentStructureBuilder},
        payload::dto::MilestonePayloadDto,
        protocol::ProtocolParameters,
        BlockDto,
    },
};

/// Response of GET /api/core/v2/info.
/// Returns general information about the node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub status: StatusResponse,
    pub supported_protocol_versions: Vec<u8>,
    pub protocol: ProtocolResponse,
    pub pending_protocol_parameters: Vec<PendingProtocolParameter>,
    pub base_token: BaseTokenResponse,
    pub metrics: MetricsResponse,
    pub features: Vec<String>,
}

/// Returned in [`InfoResponse`].
/// Status information about the node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct StatusResponse {
    pub is_healthy: bool,
    pub latest_milestone: LatestMilestoneResponse,
    pub confirmed_milestone: ConfirmedMilestoneResponse,
    pub pruning_index: u32,
}

/// Returned in [`StatusResponse`].
/// Information about the latest milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct LatestMilestoneResponse {
    pub index: u32,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub timestamp: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub milestone_id: Option<String>,
}

/// Returned in [`StatusResponse`].
/// Information about the confirmed milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct ConfirmedMilestoneResponse {
    pub index: u32,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub timestamp: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub milestone_id: Option<String>,
}

/// Returned in [`InfoResponse`].
/// Protocol information about the node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct ProtocolResponse {
    pub version: u8,
    pub network_name: String,
    pub bech32_hrp: String,
    pub min_pow_score: u32,
    pub below_max_depth: u8,
    pub rent_structure: RentStructureResponse,
    pub token_supply: String,
}

impl TryFrom<ProtocolResponse> for ProtocolParameters {
    type Error = Error;

    fn try_from(response: ProtocolResponse) -> Result<Self, Self::Error> {
        Ok(ProtocolParameters::new(
            response.version,
            response.network_name,
            response.bech32_hrp,
            response.min_pow_score,
            response.below_max_depth,
            response.rent_structure.into(),
            response
                .token_supply
                .parse()
                .map_err(|_| Error::InvalidField("token_supply"))?,
        )?)
    }
}

/// Returned in [`InfoResponse`].
/// Pending protocol parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct PendingProtocolParameter {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: u8,
    pub target_milestone_index: u32,
    pub protocol_version: u8,
    pub params: String,
}

/// Returned in [`InfoResponse`].
/// Information about the base token.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct BaseTokenResponse {
    pub name: String,
    pub ticker_symbol: String,
    pub unit: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub subunit: Option<String>,
    pub decimals: u8,
    pub use_metric_prefix: bool,
}

/// Returned in [`InfoResponse`].
/// Rent information about the node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct RentStructureResponse {
    pub v_byte_cost: u32,
    pub v_byte_factor_key: u8,
    pub v_byte_factor_data: u8,
}

impl From<RentStructureResponse> for RentStructure {
    fn from(response: RentStructureResponse) -> Self {
        RentStructureBuilder::new()
            .byte_cost(response.v_byte_cost)
            .key_factor(response.v_byte_factor_key)
            .data_factor(response.v_byte_factor_data)
            .finish()
    }
}

/// Returned in [`InfoResponse`].
/// Metric information about the node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MetricsResponse {
    pub blocks_per_second: f64,
    pub referenced_blocks_per_second: f64,
    pub referenced_rate: f64,
}

/// Response of GET /api/core/v2/tips.
/// Returns non-lazy tips.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TipsResponse {
    pub tips: Vec<String>,
}

/// Response of POST /api/core/v2/blocks.
/// Returns the block identifier of the submitted block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct SubmitBlockResponse {
    pub block_id: String,
}

/// Response of GET /api/core/v2/blocks/{block_id}.
/// Returns a specific block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase"),
    serde(untagged)
)]
pub enum BlockResponse {
    Json(BlockDto),
    Raw(Vec<u8>),
}

/// Response of GET /api/core/v2/blocks/{block_id}/metadata.
/// Returns the metadata of a block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct BlockMetadataResponse {
    pub block_id: String,
    pub parents: Vec<String>,
    pub is_solid: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub referenced_by_milestone_index: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub milestone_index: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ledger_inclusion_state: Option<LedgerInclusionStateDto>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub conflict_reason: Option<u8>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub white_flag_index: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub should_promote: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub should_reattach: Option<bool>,
}

/// Response of GET /api/core/v2/outputs/{output_id}.
/// Returns an output and its metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct OutputResponse {
    pub metadata: OutputMetadataResponse,
    pub output: OutputDto,
}

/// Response of GET /api/core/v2/outputs/{output_id}/metadata.
/// Returns an output metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct OutputMetadataResponse {
    pub block_id: String,
    pub transaction_id: String,
    pub output_index: u16,
    pub is_spent: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub milestone_index_spent: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub milestone_timestamp_spent: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transaction_id_spent: Option<String>,
    pub milestone_index_booked: u32,
    pub milestone_timestamp_booked: u32,
    pub ledger_index: u32,
}

/// Response of:
/// * GET /api/core/v2/receipts/{milestone_index}, returns all stored receipts for the given milestone index.
/// * GET /api/core/v2/receipts, returns all stored receipts, independent of a milestone index.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct ReceiptsResponse {
    pub receipts: Vec<ReceiptDto>,
}

/// Response of GET /api/core/v2/treasury.
/// Returns all information about the treasury.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct TreasuryResponse {
    pub milestone_id: String,
    pub amount: String,
}

/// Response of GET /api/core/v2/milestone/{milestone_index}.
/// Returns information about a milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase"),
    serde(untagged)
)]
pub enum MilestoneResponse {
    Json(MilestonePayloadDto),
    Raw(Vec<u8>),
}

/// Response of GET /api/core/v2/milestone/{milestone_index}/utxo-changes.
/// Returns all UTXO changes that happened at a specific milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct UtxoChangesResponse {
    pub index: u32,
    pub created_outputs: Vec<String>,
    pub consumed_outputs: Vec<String>,
}

/// Response of GET /api/core/v2/peers.
/// Returns information about all peers of the node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct PeersResponse(pub Vec<PeerDto>);

/// Response of POST /api/core/v2/peers.
/// Returns information about the added peer.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct AddPeerResponse(pub PeerDto);

/// Response of GET /api/core/v2/peer/{peer_id}.
/// Returns information about a specific peer of the node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct PeerResponse(pub PeerDto);

/// Response of GET /api/plugins/debug/whiteflag.
/// Returns the computed merkle tree hash for the given white flag traversal.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct WhiteFlagResponse {
    pub merkle_tree_hash: String,
}

/// Response of GET /api/routes.
/// Returns the available API route groups of the node.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct RoutesResponse {
    pub routes: Vec<String>,
}
