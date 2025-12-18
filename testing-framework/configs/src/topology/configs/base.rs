use thiserror::Error;

use super::{
    blend, bootstrap, bootstrap::SHORT_PROLONGED_BOOTSTRAP_PERIOD, consensus,
    consensus::ConsensusParams, da, da::DaParams, network, network::NetworkParams,
    wallet::WalletConfig,
};

#[derive(Debug, Error)]
pub enum BaseConfigError {
    #[error(transparent)]
    Consensus(#[from] consensus::ConsensusConfigError),
    #[error(transparent)]
    Da(#[from] da::DaConfigError),
    #[error(transparent)]
    Network(#[from] network::NetworkConfigError),
}

pub struct BaseConfigs {
    pub consensus_configs: Vec<consensus::GeneralConsensusConfig>,
    pub bootstrap_configs: Vec<bootstrap::GeneralBootstrapConfig>,
    pub da_configs: Vec<da::GeneralDaConfig>,
    pub network_configs: Vec<network::GeneralNetworkConfig>,
    pub blend_configs: Vec<blend::GeneralBlendConfig>,
}

pub fn build_base_configs(
    ids: &[[u8; 32]],
    consensus_params: &ConsensusParams,
    da_params: &DaParams,
    network_params: &NetworkParams,
    wallet_config: &WalletConfig,
    da_ports: &[u16],
    blend_ports: &[u16],
) -> Result<BaseConfigs, BaseConfigError> {
    Ok(BaseConfigs {
        consensus_configs: consensus::create_consensus_configs(
            ids,
            consensus_params,
            wallet_config,
        )?,
        bootstrap_configs: bootstrap::create_bootstrap_configs(
            ids,
            SHORT_PROLONGED_BOOTSTRAP_PERIOD,
        ),
        da_configs: da::try_create_da_configs(ids, da_params, da_ports)?,
        network_configs: network::create_network_configs(ids, network_params)?,
        blend_configs: blend::create_blend_configs(ids, blend_ports),
    })
}
