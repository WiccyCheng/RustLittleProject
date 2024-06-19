use anyhow::Result;
use libp2p::{
    core::transport::upgrade,
    floodsub::{self, Floodsub, Topic},
    futures::StreamExt,
    identity, mdns, noise,
    swarm::{Config, NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm, Transport,
};
use std::borrow::Cow;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

/// 处理 p2p 网络的 behavior 数据结构
/// 里面的每个域需要实现 NetworkBehaviour，或者使用 #[behaviour(ignore)]
#[derive(NetworkBehaviour)]
struct ChatBehavior {
    /// flood subscription，比较浪费带宽，gossipsub 是更好的选择
    floodsub: Floodsub,
    /// 本地节点发现机制
    mdns: mdns::tokio::Behaviour,
}

impl ChatBehavior {
    /// 创建一个新的 ChatBehavior
    pub async fn new(id: PeerId) -> Result<Self> {
        Ok(Self {
            floodsub: Floodsub::new(id),
            mdns: mdns::tokio::Behaviour::new(Default::default(), id)?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let name = match std::env::args().nth(1) {
        Some(arg) => Cow::Owned(arg),
        None => Cow::Borrowed("lobby"),
    };

    let topic = floodsub::Topic::new(name);

    let mut swarm = create_swarm(topic.clone()).await?;

    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;

    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                swarm.behaviour_mut().floodsub.publish(topic.clone(), line);
        }
         event = swarm.select_next_some() => match event {
            SwarmEvent::NewListenAddr { address, .. } => { println!("Listening on {:?}", address); },
            SwarmEvent::Behaviour(ChatBehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                // 把 mdns 发现的新的 peer 加入到 floodsub 的 view 中
                for (peer_id, multiaddr) in list {
                    println!("mDNS discovered a new peer: {peer_id} with addr {multiaddr}");
                    swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer_id);
                }
            },
            SwarmEvent::Behaviour(ChatBehaviorEvent::Mdns(mdns::Event::Expired(list))) => {
                // 把 mdns 发现的离开的 peer 加入到 floodsub 的 view 中
                for (peer_id, multiaddr) in list {
                    println!("mDNS discover peer has expired: {peer_id} with addr {multiaddr}");
                    swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&peer_id);
                }
            },
            SwarmEvent::Behaviour(ChatBehaviorEvent::Floodsub(floodsub::FloodsubEvent::Message(msg))) => {
                let text = String::from_utf8_lossy(&msg.data);
                println!("{:?}: {:?}", msg.source, text);
            }
            _ => {},
         } }
    }
}

async fn create_swarm(topic: Topic) -> Result<Swarm<ChatBehavior>> {
    // 创建 identity（密钥对）
    let id_keys = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", local_peer_id);
    // 使用 noise protocol 来处理加密和认证
    let noise_keys = noise::Config::new(&id_keys).unwrap();
    // 创建传输层
    let transport = tcp::tokio::Transport::new(tcp::Config::new().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_keys)
        .multiplex(yamux::Config::default())
        .boxed();
    // 创建 chat behavior
    let mut behaviour = ChatBehavior::new(local_peer_id.clone()).await?;
    // 订阅某个主题
    behaviour.floodsub.subscribe(topic.clone());
    // 创建 swarm
    let swarm = Swarm::new(
        transport,
        behaviour,
        local_peer_id,
        Config::with_executor(Box::new(|fut| {
            tokio::spawn(fut);
        })),
    );
    Ok(swarm)
}
