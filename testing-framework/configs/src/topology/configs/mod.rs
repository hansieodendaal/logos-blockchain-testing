pub mod api;
pub mod base;
pub mod blend;
pub mod bootstrap;
pub mod consensus;
pub mod da;
pub mod network;
pub mod time;
pub mod tracing;
pub mod wallet;

use std::cmp;

use blend::GeneralBlendConfig;
use consensus::{
    ConsensusConfigError, GeneralConsensusConfig, ProviderInfo, create_genesis_tx_with_declarations,
};
use da::GeneralDaConfig;
use key_management_system_service::{backend::preload::PreloadKMSBackendSettings, keys::Key};
use network::GeneralNetworkConfig;
use nomos_core::{
    mantle::GenesisTx as _,
    sdp::{Locator, ServiceType},
};
use nomos_utils::net::get_available_udp_port;
use rand::{Rng as _, thread_rng};
use tracing::GeneralTracingConfig;
use wallet::WalletConfig;

use crate::{
    nodes::kms::key_id_for_preload_backend,
    topology::{
        configs::{
            api::GeneralApiConfig,
            bootstrap::{GeneralBootstrapConfig, SHORT_PROLONGED_BOOTSTRAP_PERIOD},
            consensus::ConsensusParams,
            da::DaParams,
            network::NetworkParams,
            time::GeneralTimeConfig,
        },
        invariants::validate_generated_vectors,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum GeneralConfigError {
    #[error("participant count must be > 0")]
    EmptyParticipants,
    #[error("blend core subset {blend_core} exceeds participants {participants}")]
    BlendCoreSubsetTooLarge {
        blend_core: usize,
        participants: usize,
    },
    #[error("failed to allocate a free UDP port for {label}")]
    PortAllocationFailed { label: &'static str },
    #[error(transparent)]
    Invariants(#[from] crate::topology::invariants::TopologyInvariantError),
    #[error(transparent)]
    Consensus(#[from] ConsensusConfigError),
    #[error(transparent)]
    Network(#[from] network::NetworkConfigError),
    #[error(transparent)]
    Da(#[from] da::DaConfigError),
    #[error(transparent)]
    Api(#[from] api::ApiConfigError),
}

#[derive(Clone)]
pub struct GeneralConfig {
    pub api_config: GeneralApiConfig,
    pub consensus_config: GeneralConsensusConfig,
    pub bootstrapping_config: GeneralBootstrapConfig,
    pub da_config: GeneralDaConfig,
    pub network_config: GeneralNetworkConfig,
    pub blend_config: GeneralBlendConfig,
    pub tracing_config: GeneralTracingConfig,
    pub time_config: GeneralTimeConfig,
    pub kms_config: PreloadKMSBackendSettings,
}

pub fn create_general_configs(n_nodes: usize) -> Result<Vec<GeneralConfig>, GeneralConfigError> {
    create_general_configs_with_network(n_nodes, &NetworkParams::default())
}

pub fn create_general_configs_with_network(
    n_nodes: usize,
    network_params: &NetworkParams,
) -> Result<Vec<GeneralConfig>, GeneralConfigError> {
    create_general_configs_with_blend_core_subset(n_nodes, n_nodes, network_params)
}

pub fn create_general_configs_with_blend_core_subset(
    n_nodes: usize,
    // TODO: Instead of this, define a config struct for each node.
    // That would be also useful for non-even token distributions: https://github.com/logos-co/nomos/issues/1888
    n_blend_core_nodes: usize,
    network_params: &NetworkParams,
) -> Result<Vec<GeneralConfig>, GeneralConfigError> {
    if n_nodes == 0 {
        return Err(GeneralConfigError::EmptyParticipants);
    }
    if n_blend_core_nodes > n_nodes {
        return Err(GeneralConfigError::BlendCoreSubsetTooLarge {
            blend_core: n_blend_core_nodes,
            participants: n_nodes,
        });
    }

    // Blend relies on each node declaring a different ZK public key, so we need
    // different IDs to generate different keys.
    let mut ids: Vec<_> = (0..n_nodes).map(|i| [i as u8; 32]).collect();
    let mut da_ports = Vec::with_capacity(n_nodes);
    let mut blend_ports = Vec::with_capacity(n_nodes);

    for id in &mut ids {
        thread_rng().fill(id);
        da_ports.push(
            get_available_udp_port()
                .ok_or(GeneralConfigError::PortAllocationFailed { label: "DA" })?,
        );
        blend_ports.push(
            get_available_udp_port()
                .ok_or(GeneralConfigError::PortAllocationFailed { label: "Blend" })?,
        );
    }

    validate_generated_vectors(n_nodes, &ids, &da_ports, &blend_ports)?;

    let consensus_params = ConsensusParams::default_for_participants(n_nodes);
    let mut consensus_configs =
        consensus::create_consensus_configs(&ids, &consensus_params, &WalletConfig::default())?;
    let bootstrap_config =
        bootstrap::create_bootstrap_configs(&ids, SHORT_PROLONGED_BOOTSTRAP_PERIOD);
    let network_configs = network::create_network_configs(&ids, network_params)?;
    let da_configs = da::try_create_da_configs(&ids, &DaParams::default(), &da_ports)?;
    let api_configs = api::create_api_configs(&ids)?;
    let blend_configs = blend::create_blend_configs(&ids, &blend_ports);
    let tracing_configs = tracing::create_tracing_configs(&ids);
    let time_config = time::default_time_config();

    let Some(first_consensus) = consensus_configs.first() else {
        return Err(GeneralConfigError::EmptyParticipants);
    };

    let mut providers = Vec::with_capacity(cmp::min(n_blend_core_nodes, blend_configs.len()));
    for (i, blend_conf) in blend_configs
        .iter()
        .enumerate()
        .take(cmp::min(n_blend_core_nodes, blend_configs.len()))
    {
        let note = first_consensus
            .blend_notes
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        providers.push(ProviderInfo {
            service_type: ServiceType::BlendNetwork,
            provider_sk: blend_conf.signer.clone(),
            zk_sk: blend_conf.secret_zk_key.clone(),
            locator: Locator(blend_conf.backend_core.listening_address.clone()),
            note,
        });
    }
    let ledger_tx = first_consensus.genesis_tx.mantle_tx().ledger_tx.clone();
    let genesis_tx = create_genesis_tx_with_declarations(ledger_tx, providers)?;
    for c in &mut consensus_configs {
        c.genesis_tx = genesis_tx.clone();
    }

    // Set Blend and DA keys in KMS of each node config.
    let kms_configs: Vec<_> = blend_configs
        .iter()
        .map(|blend_conf| {
            let ed_key = blend_conf.signer.clone();
            let zk_key = blend_conf.secret_zk_key.clone();
            PreloadKMSBackendSettings {
                keys: [
                    (
                        key_id_for_preload_backend(&Key::from(ed_key.clone())),
                        Key::from(ed_key),
                    ),
                    (
                        key_id_for_preload_backend(&Key::from(zk_key.clone())),
                        Key::from(zk_key),
                    ),
                ]
                .into(),
            }
        })
        .collect();

    let mut general_configs = Vec::with_capacity(n_nodes);

    for i in 0..n_nodes {
        let api_config = api_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let consensus_config = consensus_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let bootstrapping_config = bootstrap_config
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let da_config = da_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let network_config = network_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let blend_config = blend_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let tracing_config = tracing_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();
        let kms_config = kms_configs
            .get(i)
            .ok_or(GeneralConfigError::EmptyParticipants)?
            .clone();

        general_configs.push(GeneralConfig {
            api_config,
            consensus_config,
            bootstrapping_config,
            da_config,
            network_config,
            blend_config,
            tracing_config,
            time_config: time_config.clone(),
            kms_config,
        });
    }

    Ok(general_configs)
}
