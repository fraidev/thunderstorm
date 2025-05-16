use dashmap::DashMap;

use crate::{bitfield::Bitfield, peer::PeerAddr};

#[derive(Debug, Clone, Default)]
pub struct PeerStates {
    pub states: DashMap<PeerAddr, PeerState>,
}

impl PeerStates {
    pub fn add_if_not_seen(&self, peer: PeerAddr) {
        if !self.states.contains_key(&peer) {
            self.states.insert(peer, PeerState::default());
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerState {
    /// This is used to track if the peer is interested in us.
    pub peer_interested: bool,
    /// This is used to track the pieces the peer has.
    pub bitfield: Bitfield,
}

impl Default for PeerState {
    fn default() -> Self {
        Self {
            peer_interested: true,
            bitfield: Bitfield::new(vec![]),
        }
    }
}
