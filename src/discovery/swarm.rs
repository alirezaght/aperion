use std::any::Any;
use std::future::{Future, Pending, poll_fn};
use std::io::{Bytes, Error};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::task::{Context, Poll};
use std::thread;
use std::thread::JoinHandle;
use async_std::task::block_on;
use clap::arg;
use libp2p_core::{Multiaddr, Transport};
use libp2p_identity::{PeerId};
use libp2p_gossipsub::{Event, IdentTopic, Message, MessageAuthenticity};
use libp2p_kad::{Kademlia, KademliaEvent, QueryId, Quorum, Record};
use libp2p_kad::handler::KademliaHandler;
use libp2p_kad::record::Key;
use libp2p_kad::store::MemoryStore;
use libp2p_swarm::derive_prelude::futures::stream::StreamExt;
use libp2p_swarm::{ConnectionHandler, ConnectionId, FromSwarm, NetworkBehaviour, NetworkBehaviourAction, PollParameters, SwarmEvent, THandlerInEvent, THandlerOutEvent};
use log::info;
use crate::Args;

const NETWORK_TOPIC: &str = "Network";

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "AperionNetworkEvent")]
struct AperionNetworkBehaviour {
    kad: Kademlia<MemoryStore>,
    gossip: libp2p_gossipsub::Behaviour,
}

impl AperionNetworkBehaviour {
    fn new(local_id: PeerId) -> Self {
        let config = libp2p_kad::KademliaConfig::default();
        let kad = libp2p_kad::Kademlia::with_config(local_id, MemoryStore::new(local_id), config);
        let local_key = libp2p_identity::Keypair::generate_ed25519();
        let gossip_config = libp2p_gossipsub::Config::default();
        let gossip = libp2p_gossipsub::Behaviour::new(MessageAuthenticity::Signed(local_key), gossip_config).unwrap();
        AperionNetworkBehaviour {
            kad,
            gossip,
        }
    }
}

#[derive(Debug)]
enum AperionNetworkEvent {
    K(KademliaEvent),
    G(Event),
}


impl From<KademliaEvent> for AperionNetworkEvent {
    fn from(event: KademliaEvent) -> Self {
        Self::K(event)
    }
}

impl From<Event> for AperionNetworkEvent {
    fn from(event: Event) -> Self {
        Self::G(event)
    }
}

struct GossipMessage {
    data: Vec<u8>
}

impl event_bus::Event for GossipMessage {

}


pub fn start(args: Args) -> JoinHandle<()> {
    let d: Vec<u8> = vec![];
    return thread::spawn(|| {
        let local_id = libp2p_identity::PeerId::random();
        let behaviour = AperionNetworkBehaviour::new(local_id);
        let id_keys = libp2p_identity::Keypair::generate_ed25519();
        let noise = libp2p_noise::NoiseAuthenticated::xx(&id_keys).unwrap();
        let transport = libp2p_tcp::async_io::Transport::default().upgrade(libp2p_core::upgrade::Version::V1).authenticate(noise).multiplex(libp2p_mplex::MplexConfig::new()).boxed();
        let mut swarm = libp2p_swarm::Swarm::without_executor(transport, behaviour, local_id);
        let port = args.port.unwrap_or("6000".parse().unwrap());
        let res = swarm.listen_on(format!("/ip4/127.0.0.1/tcp/{port}").parse().unwrap()).unwrap();
        for arg in args.nodes.unwrap_or(vec![]) {
            if arg.contains(":") {
                let mut address = arg.split(":");
                let ip = address.next().unwrap();
                let port = address.next().unwrap();
                swarm.dial(format!("/ip4/{ip}/tcp/{port}").parse::<Multiaddr>().unwrap());
            } else {
                swarm.dial(format!("{arg}").parse::<Multiaddr>().unwrap());
            }
        }


        swarm.behaviour_mut().gossip.subscribe(&IdentTopic::new(NETWORK_TOPIC));

        loop {
            match block_on(swarm.select_next_some()) {
                SwarmEvent::ConnectionEstablished { .. } => {
                }
                SwarmEvent::ConnectionClosed { .. } => {}
                SwarmEvent::IncomingConnection { .. } => {}
                SwarmEvent::IncomingConnectionError { .. } => {}
                SwarmEvent::OutgoingConnectionError { .. } => {}
                SwarmEvent::BannedPeer { .. } => {}
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {address:?}");
                }
                SwarmEvent::ExpiredListenAddr { .. } => {}
                SwarmEvent::ListenerClosed { .. } => {}
                SwarmEvent::ListenerError { .. } => {}
                SwarmEvent::Dialing(_) => {}
                SwarmEvent::Behaviour(event) => {
                    match event {
                        AperionNetworkEvent::G(msg) => {
                            match msg {
                                Event::Message { message, .. } => {
                                    info!("Received: {message:?}");
                                    event_bus::dispatch_event!(NETWORK_TOPIC, &mut GossipMessage {
                                        data: message.data
                                    });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        };
    });
}