use dashmap::DashMap;

use crate::{bitfield::Bitfield, peer::PeerAddr};

#[derive(Debug, Clone)]
pub struct PeerStates {
    pub states: DashMap<PeerAddr, PeerState>,
}

impl PeerStates {
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    pub fn add_if_not_seen(&self, peer: PeerAddr) {
        if !self.states.contains_key(&peer) {
            self.states.insert(peer, PeerState::new());
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerState {
    //#[allow(dead_code)]
    //peer_id: [u8; 20],
    pub peer_interested: bool,

    // This is used to track the pieces the peer has.
    pub bitfield: Bitfield,
    //// When the peer sends us data this is used to track if we asked for it.
    //pub inflight_requests: HashSet<InflightRequest>,

    // The main channel to send requests to peer.
    //pub tx: flume::Sender<WriterRequest>,
}

impl PeerState {
    pub fn new() -> Self {
        Self {
            //peer_id,
            peer_interested: true,
            bitfield: Bitfield::new(vec![]),
            //tx,
        }
    }
}
