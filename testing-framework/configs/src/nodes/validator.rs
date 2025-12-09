use std::{collections::HashSet, path::PathBuf, time::Duration};

use nomos_da_network_core::{
    protocols::sampling::SubnetsConfig, swarm::DAConnectionPolicySettings,
};
use nomos_da_network_service::{
    NetworkConfig as DaNetworkConfig, api::http::ApiAdapterSettings,
    backends::libp2p::common::DaNetworkBackendSettings,
};
use nomos_da_sampling::{
    DaSamplingServiceSettings, backend::kzgrs::KzgrsSamplingBackendSettings,
    verifier::kzgrs::KzgrsDaVerifierSettings as SamplingVerifierSettings,
};
use nomos_da_verifier::{
    DaVerifierServiceSettings,
    backend::{kzgrs::KzgrsDaVerifierSettings, trigger::MempoolPublishTriggerConfig},
    storage::adapters::rocksdb::RocksAdapterSettings as VerifierStorageAdapterSettings,
};
use nomos_node::{
    Config as ValidatorConfig, RocksBackendSettings,
    api::backend::AxumBackendSettings as NodeAxumBackendSettings,
    config::{
        deployment::DeploymentSettings, mempool::serde::Config as MempoolConfig,
        time::serde::Config as TimeConfig,
    },
};
use nomos_sdp::SdpSettings;
use nomos_time::backends::{NtpTimeBackendSettings, ntp::async_client::NTPClientSettings};
use nomos_utils::math::NonNegativeF64;
use nomos_wallet::WalletServiceSettings;

use crate::{
    adjust_timeout,
    nodes::{
        blend::build_blend_service_config,
        common::{cryptarchia_config, cryptarchia_deployment, mempool_deployment, time_deployment},
    },
    topology::configs::{GeneralConfig, wallet::WalletAccount},
};

#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "Validator config wiring aggregates many service settings"
)]
pub fn create_validator_config(config: GeneralConfig) -> ValidatorConfig {
    let da_policy_settings = config.da_config.policy_settings.clone();
    let network_config = config.network_config.clone();
    let (blend_user_config, blend_deployment, network_deployment) =
        build_blend_service_config(&config.blend_config);
    let deployment_settings = DeploymentSettings::new_custom(
        blend_deployment,
        network_deployment,
        cryptarchia_deployment(&config),
        time_deployment(&config),
        mempool_deployment(),
    );
    ValidatorConfig {
        network: network_config,
        blend: blend_user_config,
        deployment: deployment_settings,
        cryptarchia: cryptarchia_config(&config),
        da_network: DaNetworkConfig {
            backend: DaNetworkBackendSettings {
                node_key: config.da_config.node_key,
                listening_address: config.da_config.listening_address,
                policy_settings: DAConnectionPolicySettings {
                    min_dispersal_peers: 0,
                    min_replication_peers: da_policy_settings.min_replication_peers,
                    max_dispersal_failures: da_policy_settings.max_dispersal_failures,
                    max_sampling_failures: da_policy_settings.max_sampling_failures,
                    max_replication_failures: da_policy_settings.max_replication_failures,
                    malicious_threshold: da_policy_settings.malicious_threshold,
                },
                monitor_settings: config.da_config.monitor_settings,
                balancer_interval: config.da_config.balancer_interval,
                redial_cooldown: config.da_config.redial_cooldown,
                replication_settings: config.da_config.replication_settings,
                subnets_settings: SubnetsConfig {
                    num_of_subnets: config.da_config.num_samples as usize,
                    shares_retry_limit: config.da_config.retry_shares_limit,
                    commitments_retry_limit: config.da_config.retry_commitments_limit,
                },
            },
            membership: config.da_config.membership.clone(),
            api_adapter_settings: ApiAdapterSettings {
                api_port: config.api_config.address.port(),
                is_secure: false,
            },
            subnet_refresh_interval: config.da_config.subnets_refresh_interval,
            subnet_threshold: config.da_config.num_samples as usize,
            min_session_members: config.da_config.num_samples as usize,
        },
        da_verifier: DaVerifierServiceSettings {
            share_verifier_settings: KzgrsDaVerifierSettings {
                global_params_path: config.da_config.global_params_path.clone(),
                domain_size: config.da_config.num_subnets as usize,
            },
            tx_verifier_settings: (),
            network_adapter_settings: (),
            storage_adapter_settings: VerifierStorageAdapterSettings {
                blob_storage_directory: "./".into(),
            },
            mempool_trigger_settings: MempoolPublishTriggerConfig {
                publish_threshold: NonNegativeF64::try_from(0.8).unwrap(),
                share_duration: Duration::from_secs(5),
                prune_duration: Duration::from_secs(30),
                prune_interval: Duration::from_secs(5),
            },
        },
        tracing: config.tracing_config.tracing_settings,
        http: nomos_api::ApiServiceSettings {
            backend_settings: NodeAxumBackendSettings {
                address: config.api_config.address,
                rate_limit_per_second: 10000,
                rate_limit_burst: 10000,
                max_concurrent_requests: 1000,
                ..Default::default()
            },
        },
        da_sampling: DaSamplingServiceSettings {
            sampling_settings: KzgrsSamplingBackendSettings {
                num_samples: config.da_config.num_samples,
                num_subnets: config.da_config.num_subnets,
                old_blobs_check_interval: config.da_config.old_blobs_check_interval,
                blobs_validity_duration: config.da_config.blobs_validity_duration,
            },
            share_verifier_settings: SamplingVerifierSettings {
                global_params_path: config.da_config.global_params_path,
                domain_size: config.da_config.num_subnets as usize,
            },
            commitments_wait_duration: Duration::from_secs(1),
            sdp_blob_trigger_sampling_delay: adjust_timeout(Duration::from_secs(5)),
        },
        storage: RocksBackendSettings {
            db_path: "./db".into(),
            read_only: false,
            column_family: Some("blocks".into()),
        },
        time: TimeConfig {
            backend: NtpTimeBackendSettings {
                ntp_server: config.time_config.ntp_server,
                ntp_client_settings: NTPClientSettings {
                    timeout: config.time_config.timeout,
                    listening_interface: config.time_config.interface,
                },
                update_interval: config.time_config.update_interval,
            },
            chain_start_time: config.time_config.chain_start_time,
        },
        mempool: MempoolConfig {
            // Disable mempool recovery for hermetic tests.
            recovery_path: PathBuf::new(),
        },
        sdp: SdpSettings { declaration: None },
        wallet: WalletServiceSettings {
            known_keys: {
                let mut keys = HashSet::from_iter([config.consensus_config.leader_config.pk]);
                keys.extend(
                    config
                        .consensus_config
                        .wallet_accounts
                        .iter()
                        .map(WalletAccount::public_key),
                );
                keys
            },
        },
        key_management: config.kms_config,
        testing_http: nomos_api::ApiServiceSettings {
            backend_settings: NodeAxumBackendSettings {
                address: config.api_config.testing_http_address,
                rate_limit_per_second: 10000,
                rate_limit_burst: 10000,
                max_concurrent_requests: 1000,
                ..Default::default()
            },
        },
    }
}
