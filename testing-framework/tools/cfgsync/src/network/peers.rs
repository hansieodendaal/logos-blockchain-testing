use std::str::FromStr;

use nomos_libp2p::{Multiaddr, PeerId};

use super::address::find_matching_host;
use crate::host::Host;

pub fn rewrite_initial_peers(
    templates: &[Vec<Multiaddr>],
    original_ports: &[u16],
    hosts: &[Host],
    peer_ids: &[PeerId],
) -> Vec<Vec<Multiaddr>> {
    templates
        .iter()
        .enumerate()
        .map(|(node_idx, peers)| {
            peers
                .iter()
                .filter_map(|addr| find_matching_host(addr, original_ports))
                .filter(|&peer_idx| peer_idx != node_idx)
                .map(|peer_idx| {
                    Multiaddr::from_str(&format!(
                        "/ip4/{}/udp/{}/quic-v1/p2p/{}",
                        hosts[peer_idx].ip, hosts[peer_idx].network_port, peer_ids[peer_idx]
                    ))
                    .expect("valid peer multiaddr")
                })
                .collect()
        })
        .collect()
}
