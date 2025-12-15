use std::{num::NonZeroU64, path::PathBuf, time::Duration};

use blend_serde::Config as BlendUserConfig;
use key_management_system_service::keys::Key;
use nomos_blend_service::{
    core::settings::{CoverTrafficSettings, MessageDelayerSettings, SchedulerSettings, ZkSettings},
    settings::TimingSettings,
};
use nomos_node::config::{
    blend::{
        deployment::{self as blend_deployment, Settings as BlendDeploymentSettings},
        serde as blend_serde,
    },
    network::deployment::Settings as NetworkDeploymentSettings,
};
use nomos_utils::math::NonNegativeF64;

use crate::{
    nodes::kms::key_id_for_preload_backend,
    topology::configs::blend::GeneralBlendConfig as TopologyBlendConfig,
};

// Blend service constants
const BLEND_LAYERS_COUNT: u64 = 1;
const MINIMUM_NETWORK_SIZE: u64 = 1;
const ROUND_DURATION_SECS: u64 = 1;
const ROUNDS_PER_INTERVAL: u64 = 30;
const ROUNDS_PER_SESSION: u64 = 648_000;
const ROUNDS_PER_OBSERVATION_WINDOW: u64 = 30;
const ROUNDS_PER_SESSION_TRANSITION: u64 = 30;
const EPOCH_TRANSITION_SLOTS: u64 = 2_600;
const SAFETY_BUFFER_INTERVALS: u64 = 100;
const MESSAGE_FREQUENCY_PER_ROUND: f64 = 1.0;
const MAX_RELEASE_DELAY_ROUNDS: u64 = 3;

pub(crate) fn build_blend_service_config(
    config: &TopologyBlendConfig,
) -> (
    BlendUserConfig,
    BlendDeploymentSettings,
    NetworkDeploymentSettings,
) {
    let zk_key_id = key_id_for_preload_backend(&Key::from(config.secret_zk_key.clone()));

    let backend_core = &config.backend_core;
    let backend_edge = &config.backend_edge;

    let user = BlendUserConfig {
        non_ephemeral_signing_key: config.private_key.clone(),
        // Persist recovery data under the tempdir so components expecting it
        // can start cleanly.
        recovery_path_prefix: PathBuf::from("./recovery/blend"),
        core: blend_serde::core::Config {
            backend: blend_serde::core::BackendConfig {
                listening_address: backend_core.listening_address.clone(),
                core_peering_degree: backend_core.core_peering_degree.clone(),
                edge_node_connection_timeout: backend_core.edge_node_connection_timeout,
                max_edge_node_incoming_connections: backend_core.max_edge_node_incoming_connections,
                max_dial_attempts_per_peer: backend_core.max_dial_attempts_per_peer,
            },
            zk: ZkSettings {
                secret_key_kms_id: zk_key_id,
            },
        },
        edge: blend_serde::edge::Config {
            backend: blend_serde::edge::BackendConfig {
                max_dial_attempts_per_peer_per_message: backend_edge
                    .max_dial_attempts_per_peer_per_message,
                replication_factor: backend_edge.replication_factor,
            },
        },
    };

    let deployment_settings = BlendDeploymentSettings {
        common: blend_deployment::CommonSettings {
            num_blend_layers: NonZeroU64::try_from(BLEND_LAYERS_COUNT).unwrap(),
            minimum_network_size: NonZeroU64::try_from(MINIMUM_NETWORK_SIZE).unwrap(),
            timing: TimingSettings {
                round_duration: Duration::from_secs(ROUND_DURATION_SECS),
                rounds_per_interval: NonZeroU64::try_from(ROUNDS_PER_INTERVAL).unwrap(),
                rounds_per_session: NonZeroU64::try_from(ROUNDS_PER_SESSION).unwrap(),
                rounds_per_observation_window: NonZeroU64::try_from(ROUNDS_PER_OBSERVATION_WINDOW)
                    .unwrap(),
                rounds_per_session_transition_period: NonZeroU64::try_from(
                    ROUNDS_PER_SESSION_TRANSITION,
                )
                .unwrap(),
                epoch_transition_period_in_slots: NonZeroU64::try_from(EPOCH_TRANSITION_SLOTS)
                    .unwrap(),
            },
            protocol_name: backend_core.protocol_name.clone(),
        },
        core: blend_deployment::CoreSettings {
            scheduler: SchedulerSettings {
                cover: CoverTrafficSettings {
                    intervals_for_safety_buffer: SAFETY_BUFFER_INTERVALS,
                    message_frequency_per_round: NonNegativeF64::try_from(
                        MESSAGE_FREQUENCY_PER_ROUND,
                    )
                    .unwrap(),
                },
                delayer: MessageDelayerSettings {
                    maximum_release_delay_in_rounds: NonZeroU64::try_from(MAX_RELEASE_DELAY_ROUNDS)
                        .unwrap(),
                },
            },
            minimum_messages_coefficient: backend_core.minimum_messages_coefficient,
            normalization_constant: backend_core.normalization_constant,
        },
    };

    let network_deployment = NetworkDeploymentSettings {
        identify_protocol_name: nomos_libp2p::protocol_name::StreamProtocol::new(
            "/integration/nomos/identify/1.0.0",
        ),
        kademlia_protocol_name: nomos_libp2p::protocol_name::StreamProtocol::new(
            "/integration/nomos/kad/1.0.0",
        ),
    };

    (user, deployment_settings, network_deployment)
}
