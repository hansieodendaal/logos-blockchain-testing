use std::{collections::HashSet, num::NonZeroUsize, path::PathBuf, time::Duration};

use chain_leader::LeaderConfig as ChainLeaderConfig;
use chain_network::{BootstrapConfig as ChainBootstrapConfig, OrphanConfig, SyncConfig};
use chain_service::StartingState;
use nomos_node::config::{
    cryptarchia::{
        deployment::{SdpConfig as DeploymentSdpConfig, Settings as CryptarchiaDeploymentSettings},
        serde::{
            Config as CryptarchiaConfig, LeaderConfig as CryptarchiaLeaderConfig,
            NetworkConfig as CryptarchiaNetworkConfig, ServiceConfig as CryptarchiaServiceConfig,
        },
    },
    mempool::deployment::Settings as MempoolDeploymentSettings,
    time::deployment::Settings as TimeDeploymentSettings,
};

use crate::topology::configs::GeneralConfig;

pub(crate) fn cryptarchia_deployment(config: &GeneralConfig) -> CryptarchiaDeploymentSettings {
    CryptarchiaDeploymentSettings {
        epoch_config: config.consensus_config.ledger_config.epoch_config,
        consensus_config: config.consensus_config.ledger_config.consensus_config,
        sdp_config: DeploymentSdpConfig {
            service_params: config
                .consensus_config
                .ledger_config
                .sdp_config
                .service_params
                .clone(),
            min_stake: config.consensus_config.ledger_config.sdp_config.min_stake,
        },
        gossipsub_protocol: "/cryptarchia/proto".to_owned(),
    }
}

pub(crate) fn time_deployment(config: &GeneralConfig) -> TimeDeploymentSettings {
    TimeDeploymentSettings {
        slot_duration: config.time_config.slot_duration,
    }
}

pub(crate) fn mempool_deployment() -> MempoolDeploymentSettings {
    MempoolDeploymentSettings {
        pubsub_topic: "mantle".to_owned(),
    }
}

pub(crate) fn cryptarchia_config(config: &GeneralConfig) -> CryptarchiaConfig {
    CryptarchiaConfig {
        service: CryptarchiaServiceConfig {
            starting_state: StartingState::Genesis {
                genesis_tx: config.consensus_config.genesis_tx.clone(),
            },
            // Disable on-disk recovery in compose tests to avoid serde errors on
            // non-string keys and keep services alive.
            recovery_file: PathBuf::new(),
            bootstrap: chain_service::BootstrapConfig {
                prolonged_bootstrap_period: config.bootstrapping_config.prolonged_bootstrap_period,
                force_bootstrap: false,
                offline_grace_period: chain_service::OfflineGracePeriodConfig {
                    grace_period: Duration::from_secs(20 * 60),
                    state_recording_interval: Duration::from_secs(60),
                },
            },
        },
        network: CryptarchiaNetworkConfig {
            bootstrap: ChainBootstrapConfig {
                ibd: chain_network::IbdConfig {
                    peers: HashSet::new(),
                    delay_before_new_download: Duration::from_secs(10),
                },
            },
            sync: SyncConfig {
                orphan: OrphanConfig {
                    max_orphan_cache_size: NonZeroUsize::new(5)
                        .expect("Max orphan cache size must be non-zero"),
                },
            },
        },
        leader: CryptarchiaLeaderConfig {
            leader: ChainLeaderConfig {
                pk: config.consensus_config.leader_config.pk,
                sk: config.consensus_config.leader_config.sk.clone(),
            },
        },
    }
}
