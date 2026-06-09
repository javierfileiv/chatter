use std::{ collections::HashMap, net::SocketAddr };
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use log::{ info, warn };

/// Internal broker client
#[derive(Debug, Clone)]
pub struct BrokerClient {
    /// Client username
    user: String,
    /// Address of the connected client
    addr: SocketAddr,
    /// Channel for sending messages to the client
    send_to_client: mpsc::UnboundedSender<Message>,
}

/// Internal broker events used by clients
#[derive(Debug, Clone)]
pub enum BrokerEvent {
    /// Event for when a client connects
    Connected {
        client: BrokerClient,
    },
    /// Event for when a client disconnects
    Disconnected {
        /// Address of the disconnected client
        addr: SocketAddr,
    },
    /// Event for broadcasting a message to all clients
    Broadcast {
        /// Address of the sender
        sender_addr: SocketAddr,
        /// Message to broadcast
        message: Message,
    },
    /// Event for creating a new room
    CreateRoom {
        /// Name of the room to create
        name: String,
    },
}

/// Room basic structure
struct Room {
    /// Room unique identifier
    uid: u32,
    /// Room name
    name: String,
    /// Map of clients connected to the room
    /// by IP addr and broadcast channel for the broker to send messages
    clients: Vec<BrokerClient>,
}

/// Initialize the broker and return the channel to send events to it
pub fn init() -> mpsc::UnboundedSender<BrokerEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<BrokerEvent>();
    tokio::spawn(run(rx));
    tx
}

fn create_room() {}

fn join_room() {}

fn register_client(
    user: String,
    addr: SocketAddr,
    send_to_client: mpsc::UnboundedSender<Message>
) {}

/// Run the broker event loop
pub async fn run(mut rx_events: mpsc::UnboundedReceiver<BrokerEvent>) {
    let rooms: Vec<Room> = vec![];

    while let Some(event) = rx_events.recv().await {
        match event {
            BrokerEvent::Connected { client } => {
                info!("Broker: Connected: {} {}", client.user, client.addr);
            }
            BrokerEvent::Broadcast { sender_addr, message } => {
                info!("Broker: Broadcast: {} {}", sender_addr, message);
            }
            _ => {
                warn!("event not recognised");
            }
        }
    }

    info!("Broker closed")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Petite fonction d'aide pour initialiser les logs une seule fois
    fn init_logs() {
        // Lance le logger de ton choix, par exemple env_logger :
        // let _ = env_logger::builder().is_test(true).try_init();
    }
    #[tokio::test]
    async fn test_broker_connection() {
        init_logs();
        // 1. On démarre le broker via ton init()
        let tx_broker = init();

        // 2. On crée un faux client avec son propre canal de réception
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let (tx_vers_client, mut rx_du_client) = mpsc::unbounded_channel();

        let client = BrokerClient {
            user: "Alice".to_string(),
            addr,
            send_to_client: tx_vers_client,
        };

        // 3. ACTION : On envoie l'événement de connexion au Broker
        // Si le broker tourne bien en tâche de fond, il va intercepter cet événement
        let resultat = tx_broker.send(BrokerEvent::Connected { client });

        // On vérifie que l'envoi au broker n'a pas échoué
        assert!(resultat.is_ok(), "Le Broker aurait dû recevoir l'événement Connected");
    }

    #[tokio::test]
    async fn test_broker_broadcast() {

        let tx_broker = init();
        let addr_alice: SocketAddr = "127.0.0.1:1111".parse().unwrap();
        let (tx_alice, _rx_alice) = mpsc::unbounded_channel();

        let alice = BrokerClient {
            user: "Alice".to_string(),
            addr: addr_alice,
            send_to_client: tx_alice,
        };

        // On connecte Alice
        tx_broker.send(BrokerEvent::Connected { client: alice }).unwrap();

        // On simule l'envoi d'un message de Broadcast
        let message_test = Message::Text("Hello World".into());
        tx_broker
            .send(BrokerEvent::Broadcast {
                sender_addr: addr_alice,
                message: message_test,
            })
            .unwrap();
    }
}
