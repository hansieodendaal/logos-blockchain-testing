use std::time::Duration;

use nomos_core::{
    mantle::GenesisTx as _,
    sdp::{Locator, ServiceType},
};
use nomos_da_network_core::swarm::DAConnectionPolicySettings;
use testing_framework_config::topology::{
    configs::{
        api::{ApiConfigError, create_api_configs},
        base::{BaseConfigError, BaseConfigs, build_base_configs},
        consensus::{
            ConsensusConfigError, ConsensusParams, ProviderInfo,
            create_genesis_tx_with_declarations,
        },
        da::DaParams,
        network::{Libp2pNetworkLayout, NetworkParams},
        tracing::create_tracing_configs,
        wallet::WalletConfig,
    },
    invariants::{TopologyInvariantError, validate_generated_vectors},
};
use thiserror::Error;

use crate::topology::{
    configs::{GeneralConfig, time::default_time_config},
    generation::{GeneratedNodeConfig, GeneratedTopology, NodeRole},
    utils::{TopologyResolveError, create_kms_configs, resolve_ids, resolve_ports},
};

const DEFAULT_DA_BALANCER_INTERVAL: Duration = Duration::from_secs(1);
const VALIDATOR_EXECUTOR_DA_BALANCER_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Error)]
pub enum TopologyBuildError {
    #[error("topology must include at least one node")]
    EmptyParticipants,
    #[error(transparent)]
    Invariants(#[from] TopologyInvariantError),
    #[error(transparent)]
    Resolve(#[from] TopologyResolveError),
    #[error(transparent)]
    Base(#[from] BaseConfigError),
    #[error(transparent)]
    Api(#[from] ApiConfigError),
    #[error(transparent)]
    Genesis(#[from] ConsensusConfigError),
    #[error("config generation requires at least one consensus config")]
    MissingConsensusConfig,
    #[error("internal config vector mismatch for {label} (expected {expected}, got {actual})")]
    VectorLenMismatch {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
}

/// High-level topology settings used to generate node configs for a scenario.
#[derive(Clone)]
pub struct TopologyConfig {
    pub n_validators: usize,
    pub n_executors: usize,
    pub consensus_params: ConsensusParams,
    pub da_params: DaParams,
    pub network_params: NetworkParams,
    pub wallet_config: WalletConfig,
}

impl TopologyConfig {
    /// Create a config with zero nodes; counts must be set before building.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            n_validators: 0,
            n_executors: 0,
            consensus_params: ConsensusParams::default_for_participants(1),
            da_params: DaParams::default(),
            network_params: NetworkParams::default(),
            wallet_config: WalletConfig::default(),
        }
    }

    #[must_use]
    /// Convenience config with two validators for consensus-only scenarios.
    pub fn two_validators() -> Self {
        Self {
            n_validators: 2,
            n_executors: 0,
            consensus_params: ConsensusParams::default_for_participants(2),
            da_params: DaParams::default(),
            network_params: NetworkParams::default(),
            wallet_config: WalletConfig::default(),
        }
    }

    #[must_use]
    /// Single validator + single executor config for minimal dual-role setups.
    pub fn validator_and_executor() -> Self {
        Self {
            n_validators: 1,
            n_executors: 1,
            consensus_params: ConsensusParams::default_for_participants(2),
            da_params: DaParams {
                dispersal_factor: 2,
                subnetwork_size: 2,
                num_subnets: 2,
                policy_settings: DAConnectionPolicySettings {
                    min_dispersal_peers: 1,
                    min_replication_peers: 1,
                    max_dispersal_failures: 0,
                    max_sampling_failures: 0,
                    max_replication_failures: 0,
                    malicious_threshold: 0,
                },
                balancer_interval: DEFAULT_DA_BALANCER_INTERVAL,
                ..Default::default()
            },
            network_params: NetworkParams::default(),
            wallet_config: WalletConfig::default(),
        }
    }

    #[must_use]
    /// Build a topology with explicit validator and executor counts.
    pub fn with_node_numbers(validators: usize, executors: usize) -> Self {
        let participants = validators + executors;

        let mut da_params = DaParams::default();
        let da_nodes = participants;
        if da_nodes <= 1 {
            da_params.subnetwork_size = 1;
            da_params.num_subnets = 1;
            da_params.dispersal_factor = 1;
            da_params.policy_settings.min_dispersal_peers = 0;
            da_params.policy_settings.min_replication_peers = 0;
        } else {
            let dispersal = da_nodes.min(da_params.dispersal_factor.max(2));
            da_params.dispersal_factor = dispersal;
            da_params.subnetwork_size = da_params.subnetwork_size.max(dispersal);
            da_params.num_subnets = da_params.subnetwork_size as u16;
            let min_peers = dispersal.saturating_sub(1).max(1);
            da_params.policy_settings.min_dispersal_peers = min_peers;
            da_params.policy_settings.min_replication_peers = min_peers;
            da_params.balancer_interval = DEFAULT_DA_BALANCER_INTERVAL;
        }

        Self {
            n_validators: validators,
            n_executors: executors,
            consensus_params: ConsensusParams::default_for_participants(participants),
            da_params,
            network_params: NetworkParams::default(),
            wallet_config: WalletConfig::default(),
        }
    }

    #[must_use]
    /// Build a topology with one executor and a configurable validator set.
    pub fn validators_and_executor(
        num_validators: usize,
        num_subnets: usize,
        dispersal_factor: usize,
    ) -> Self {
        Self {
            n_validators: num_validators,
            n_executors: 1,
            consensus_params: ConsensusParams::default_for_participants(num_validators + 1),
            da_params: DaParams {
                dispersal_factor,
                subnetwork_size: num_subnets,
                num_subnets: num_subnets as u16,
                policy_settings: DAConnectionPolicySettings {
                    min_dispersal_peers: num_subnets,
                    min_replication_peers: dispersal_factor - 1,
                    max_dispersal_failures: 0,
                    max_sampling_failures: 0,
                    max_replication_failures: 0,
                    malicious_threshold: 0,
                },
                balancer_interval: VALIDATOR_EXECUTOR_DA_BALANCER_INTERVAL,
                ..Default::default()
            },
            network_params: NetworkParams::default(),
            wallet_config: WalletConfig::default(),
        }
    }

    #[must_use]
    pub const fn wallet(&self) -> &WalletConfig {
        &self.wallet_config
    }
}

/// Builder that produces `GeneratedTopology` instances from a `TopologyConfig`.
#[derive(Clone)]
pub struct TopologyBuilder {
    config: TopologyConfig,
    ids: Option<Vec<[u8; 32]>>,
    da_ports: Option<Vec<u16>>,
    blend_ports: Option<Vec<u16>>,
}

impl TopologyBuilder {
    #[must_use]
    /// Create a builder from a base topology config.
    pub const fn new(config: TopologyConfig) -> Self {
        Self {
            config,
            ids: None,
            da_ports: None,
            blend_ports: None,
        }
    }

    #[must_use]
    /// Provide deterministic node IDs.
    pub fn with_ids(mut self, ids: Vec<[u8; 32]>) -> Self {
        self.ids = Some(ids);
        self
    }

    #[must_use]
    /// Override DA ports for nodes in order.
    pub fn with_da_ports(mut self, ports: Vec<u16>) -> Self {
        self.da_ports = Some(ports);
        self
    }

    #[must_use]
    /// Override blend ports for nodes in order.
    pub fn with_blend_ports(mut self, ports: Vec<u16>) -> Self {
        self.blend_ports = Some(ports);
        self
    }

    #[must_use]
    pub const fn with_validator_count(mut self, validators: usize) -> Self {
        self.config.n_validators = validators;
        self
    }

    #[must_use]
    /// Set executor count.
    pub const fn with_executor_count(mut self, executors: usize) -> Self {
        self.config.n_executors = executors;
        self
    }

    #[must_use]
    /// Set validator and executor counts together.
    pub const fn with_node_counts(mut self, validators: usize, executors: usize) -> Self {
        self.config.n_validators = validators;
        self.config.n_executors = executors;
        self
    }

    #[must_use]
    /// Configure the libp2p network layout.
    pub const fn with_network_layout(mut self, layout: Libp2pNetworkLayout) -> Self {
        self.config.network_params.libp2p_network_layout = layout;
        self
    }

    /// Override wallet configuration used in genesis.
    pub fn with_wallet_config(mut self, wallet: WalletConfig) -> Self {
        self.config.wallet_config = wallet;
        self
    }

    /// Finalize and generate topology and node descriptors.
    pub fn build(self) -> Result<GeneratedTopology, TopologyBuildError> {
        let Self {
            config,
            ids,
            da_ports,
            blend_ports,
        } = self;

        let n_participants = config.n_validators + config.n_executors;
        if n_participants == 0 {
            return Err(TopologyBuildError::EmptyParticipants);
        }

        let ids = resolve_ids(ids, n_participants)?;
        let da_ports = resolve_ports(da_ports, n_participants, "DA")?;
        let blend_ports = resolve_ports(blend_ports, n_participants, "Blend")?;

        validate_generated_vectors(n_participants, &ids, &da_ports, &blend_ports)?;

        let BaseConfigs {
            mut consensus_configs,
            bootstrap_configs: bootstrapping_config,
            da_configs,
            network_configs,
            blend_configs,
        } = build_base_configs(
            &ids,
            &config.consensus_params,
            &config.da_params,
            &config.network_params,
            &config.wallet_config,
            &da_ports,
            &blend_ports,
        )?;
        let api_configs = create_api_configs(&ids)?;
        let tracing_configs = create_tracing_configs(&ids);
        let time_config = default_time_config();

        let first_consensus = consensus_configs
            .first()
            .ok_or(TopologyBuildError::MissingConsensusConfig)?;
        let mut providers = Vec::with_capacity(da_configs.len() + blend_configs.len());
        for (i, da_conf) in da_configs.iter().enumerate() {
            let note = first_consensus
                .da_notes
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "da_notes",
                    expected: da_configs.len(),
                    actual: first_consensus.da_notes.len(),
                })?
                .clone();
            providers.push(ProviderInfo {
                service_type: ServiceType::DataAvailability,
                provider_sk: da_conf.signer.clone(),
                zk_sk: da_conf.secret_zk_key.clone(),
                locator: Locator(da_conf.listening_address.clone()),
                note,
            });
        }
        for (i, blend_conf) in blend_configs.iter().enumerate() {
            let note = first_consensus
                .blend_notes
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "blend_notes",
                    expected: blend_configs.len(),
                    actual: first_consensus.blend_notes.len(),
                })?
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

        let kms_configs =
            create_kms_configs(&blend_configs, &da_configs, &config.wallet_config.accounts);

        let mut validators = Vec::with_capacity(config.n_validators);
        let mut executors = Vec::with_capacity(config.n_executors);

        for i in 0..n_participants {
            let consensus_config = consensus_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "consensus_configs",
                    expected: n_participants,
                    actual: consensus_configs.len(),
                })?
                .clone();
            let bootstrapping_config = bootstrapping_config
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "bootstrap_configs",
                    expected: n_participants,
                    actual: bootstrapping_config.len(),
                })?
                .clone();
            let da_config = da_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "da_configs",
                    expected: n_participants,
                    actual: da_configs.len(),
                })?
                .clone();
            let network_config = network_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "network_configs",
                    expected: n_participants,
                    actual: network_configs.len(),
                })?
                .clone();
            let blend_config = blend_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "blend_configs",
                    expected: n_participants,
                    actual: blend_configs.len(),
                })?
                .clone();
            let api_config = api_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "api_configs",
                    expected: n_participants,
                    actual: api_configs.len(),
                })?
                .clone();
            let tracing_config = tracing_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "tracing_configs",
                    expected: n_participants,
                    actual: tracing_configs.len(),
                })?
                .clone();
            let kms_config = kms_configs
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "kms_configs",
                    expected: n_participants,
                    actual: kms_configs.len(),
                })?
                .clone();
            let id = *ids.get(i).ok_or(TopologyBuildError::VectorLenMismatch {
                label: "ids",
                expected: n_participants,
                actual: ids.len(),
            })?;
            let da_port = *da_ports
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "da_ports",
                    expected: n_participants,
                    actual: da_ports.len(),
                })?;
            let blend_port = *blend_ports
                .get(i)
                .ok_or(TopologyBuildError::VectorLenMismatch {
                    label: "blend_ports",
                    expected: n_participants,
                    actual: blend_ports.len(),
                })?;

            let general = GeneralConfig {
                consensus_config,
                bootstrapping_config,
                da_config,
                network_config,
                blend_config,
                api_config,
                tracing_config,
                time_config: time_config.clone(),
                kms_config,
            };

            let role = if i < config.n_validators {
                NodeRole::Validator
            } else {
                NodeRole::Executor
            };
            let index = match role {
                NodeRole::Validator => i,
                NodeRole::Executor => i - config.n_validators,
            };

            let descriptor = GeneratedNodeConfig {
                role,
                index,
                id,
                general,
                da_port,
                blend_port,
            };

            match role {
                NodeRole::Validator => validators.push(descriptor),
                NodeRole::Executor => executors.push(descriptor),
            }
        }

        Ok(GeneratedTopology {
            config,
            validators,
            executors,
        })
    }

    #[must_use]
    pub const fn config(&self) -> &TopologyConfig {
        &self.config
    }
}
