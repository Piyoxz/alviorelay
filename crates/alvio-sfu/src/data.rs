use alvio_core::PeerId;
use bytes::Bytes;
use dashmap::DashMap;
use std::collections::HashSet;
use tracing::debug;

/// Reliability profile for a WebRTC DataChannel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataChannelReliability {
    /// Ordered delivery with guaranteed retransmissions (e.g. chat, room state, control).
    Reliable,
    /// Unordered delivery with bounded retransmits (e.g. mouse cursor, game position).
    UnreliableLossy { max_retransmits: u16 },
    /// Unordered delivery with bounded packet lifetime in milliseconds (e.g. sensor/telemetry).
    UnreliableTimed { lifetime_ms: u16 },
}

/// Metadata and configuration for an active DataChannel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataChannelMetadata {
    pub label: String,
    pub reliability: DataChannelReliability,
    pub ordered: bool,
}

impl DataChannelMetadata {
    pub fn reliable(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            reliability: DataChannelReliability::Reliable,
            ordered: true,
        }
    }

    pub fn lossy(label: impl Into<String>, max_retransmits: u16) -> Self {
        Self {
            label: label.into(),
            reliability: DataChannelReliability::UnreliableLossy { max_retransmits },
            ordered: false,
        }
    }

    pub fn timed(label: impl Into<String>, lifetime_ms: u16) -> Self {
        Self {
            label: label.into(),
            reliability: DataChannelReliability::UnreliableTimed { lifetime_ms },
            ordered: false,
        }
    }
}

/// Message transmitted across a WebRTC DataChannel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMessage {
    pub sender_peer_id: PeerId,
    pub label: String,
    pub binary: bool,
    pub payload: Bytes,
}

impl DataMessage {
    pub fn text(sender: PeerId, label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            sender_peer_id: sender,
            label: label.into(),
            binary: false,
            payload: Bytes::from(text.into()),
        }
    }

    pub fn binary(sender: PeerId, label: impl Into<String>, bytes: impl Into<Bytes>) -> Self {
        Self {
            sender_peer_id: sender,
            label: label.into(),
            binary: true,
            payload: bytes.into(),
        }
    }
}

/// Lock-free Router for WebRTC Data Channels within a Room.
///
/// Dispatches broadcast messages across subscribed peers and directs unicast data
/// with zero allocations on the read hot path.
pub struct DataRouter {
    peers: DashMap<PeerId, HashSet<String>>,
    declared_channels: DashMap<String, DataChannelMetadata>,
}

impl DataRouter {
    pub fn new() -> Self {
        Self {
            peers: DashMap::new(),
            declared_channels: DashMap::new(),
        }
    }

    /// Registers a peer in the data router.
    pub fn register_peer(&self, peer_id: PeerId) {
        debug!(%peer_id, "Registered peer in DataRouter");
        self.peers.entry(peer_id).or_default();
    }

    /// Unregisters a peer and removes their channel subscriptions.
    pub fn unregister_peer(&self, peer_id: &PeerId) {
        debug!(%peer_id, "Unregistered peer from DataRouter");
        self.peers.remove(peer_id);
    }

    /// Declares a global data channel topic/label with its reliability profile.
    pub fn declare_channel(&self, metadata: DataChannelMetadata) {
        self.declared_channels
            .insert(metadata.label.clone(), metadata);
    }

    /// Subscribes a peer to a specific data channel label (e.g. "chat", "cursor").
    pub fn subscribe(&self, peer_id: &PeerId, label: &str) {
        if let Some(mut set) = self.peers.get_mut(peer_id) {
            set.insert(label.to_string());
        }
    }

    /// Unsubscribes a peer from a data channel label.
    pub fn unsubscribe(&self, peer_id: &PeerId, label: &str) {
        if let Some(mut set) = self.peers.get_mut(peer_id) {
            set.remove(label);
        }
    }

    /// Dispatches a broadcast data message to all eligible peers in the room.
    ///
    /// Returns the list of recipient `PeerId`s who are actively subscribed
    /// to `message.label`, excluding the sender.
    pub fn route_broadcast(&self, message: &DataMessage) -> Vec<PeerId> {
        let mut recipients = Vec::new();

        for peer_entry in self.peers.iter() {
            let peer_id = peer_entry.key();
            if *peer_id != message.sender_peer_id {
                let subscriptions = peer_entry.value();
                // If peer is subscribed to this label, or if peer has no explicit filters (all channels)
                if subscriptions.is_empty() || subscriptions.contains(&message.label) {
                    recipients.push(peer_id.clone());
                }
            }
        }

        recipients
    }

    /// Evaluates if a unicast direct message can be delivered to the target peer.
    pub fn route_direct(&self, message: &DataMessage, recipient: &PeerId) -> bool {
        if let Some(subscriptions) = self.peers.get(recipient) {
            subscriptions.is_empty() || subscriptions.contains(&message.label)
        } else {
            false
        }
    }

    pub fn active_peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn subscriber_count(&self, label: &str) -> usize {
        self.peers
            .iter()
            .filter(|e| e.value().is_empty() || e.value().contains(label))
            .count()
    }
}

impl Default for DataRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_channel_metadata_profiles() {
        let chat = DataChannelMetadata::reliable("chat");
        assert_eq!(chat.reliability, DataChannelReliability::Reliable);
        assert!(chat.ordered);

        let cursor = DataChannelMetadata::lossy("cursor", 0);
        assert_eq!(
            cursor.reliability,
            DataChannelReliability::UnreliableLossy { max_retransmits: 0 }
        );
        assert!(!cursor.ordered);
    }

    #[test]
    fn test_data_router_broadcast_and_isolation() {
        let router = DataRouter::new();

        let alice = PeerId::from("alice");
        let bob = PeerId::from("bob");
        let charlie = PeerId::from("charlie");

        router.register_peer(alice.clone());
        router.register_peer(bob.clone());
        router.register_peer(charlie.clone());

        // Alice and Bob subscribe to "chat"
        router.subscribe(&alice, "chat");
        router.subscribe(&bob, "chat");

        // Charlie only subscribes to "cursor"
        router.subscribe(&charlie, "cursor");

        // Alice sends a message on "chat"
        let msg = DataMessage::text(alice.clone(), "chat", "Hello Bob!");
        let recipients = router.route_broadcast(&msg);

        // Bob should receive it; Alice (sender) and Charlie (wrong topic) should not
        assert_eq!(recipients, vec![bob.clone()]);

        // Charlie sends a cursor update
        let cursor_msg = DataMessage::binary(charlie.clone(), "cursor", vec![0x01, 0x02]);
        let cursor_recipients = router.route_broadcast(&cursor_msg);

        // Neither Alice nor Bob subscribed to cursor -> empty recipients
        assert!(cursor_recipients.is_empty());

        // Now Bob subscribes to "cursor" as well
        router.subscribe(&bob, "cursor");
        let cursor_recipients_2 = router.route_broadcast(&cursor_msg);
        assert_eq!(cursor_recipients_2, vec![bob.clone()]);
    }

    #[test]
    fn test_data_router_direct_unicast() {
        let router = DataRouter::new();

        let alice = PeerId::from("alice");
        let bob = PeerId::from("bob");

        router.register_peer(alice.clone());
        router.register_peer(bob.clone());
        router.subscribe(&bob, "dm");

        let msg = DataMessage::text(alice, "dm", "Private message to Bob");

        assert!(router.route_direct(&msg, &bob));
        assert!(!router.route_direct(&msg, &PeerId::from("unknown-peer")));
    }
}
