mod craft;
mod gather;
mod persistence;

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use craft::{all_recipes, apply_craft, can_craft};
use gather::{all_gather_actions, apply_gather};
use futures_util::{SinkExt, StreamExt};
use killian_protocol::{CharacterData, ChatLine, ClientMsg, InventoryItem, ServerMsg};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type WsWriter = futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>;
type SharedState = Arc<Mutex<HashSet<String>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("KILLIAN_BIND").ok())
        .or_else(|| std::env::var("CHAT_BIND").ok())
        .unwrap_or_else(|| "0.0.0.0:7001".to_string())
        .parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("killian-server online em {}", addr);

    let (bus_tx, _bus_rx) = broadcast::channel::<ServerMsg>(512);
    let active_nicks: SharedState = Arc::new(Mutex::new(HashSet::new()));

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let bus_tx = bus_tx.clone();
        let bus_rx = bus_tx.subscribe();
        let active_nicks = active_nicks.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, peer_addr, bus_tx, bus_rx, active_nicks).await {
                eprintln!("erro cliente {}: {err}", peer_addr);
            }
        });
    }
}

fn initial_inventory() -> Vec<InventoryItem> {
    vec![
        InventoryItem { name: "Pocao Pequena".to_string(), qty: 3 },
        InventoryItem { name: "Espada Curta".to_string(), qty: 1 },
        InventoryItem { name: "Madeira".to_string(), qty: 12 },
        InventoryItem { name: "Pedra".to_string(), qty: 6 },
    ]
}

fn initial_character() -> CharacterData {
    CharacterData {
        class_name: "Aventureiro".to_string(),
        level: 1,
        hp: 100,
        max_hp: 100,
        mp: 35,
        max_mp: 35,
        gold: 150,
    }
}

fn broadcast_players(bus_tx: &broadcast::Sender<ServerMsg>, active_nicks: &SharedState) {
    let mut players: Vec<String> = active_nicks.lock().unwrap().iter().cloned().collect();
    players.sort();
    let _ = bus_tx.send(ServerMsg::PlayersUpdate { players });
}

async fn send_msg(writer: &mut WsWriter, msg: &ServerMsg) -> anyhow::Result<()> {
    let payload = serde_json::to_string(msg)?;
    writer.send(Message::Text(payload.into())).await?;
    Ok(())
}

async fn handle_client(
    stream: TcpStream,
    peer_addr: SocketAddr,
    bus_tx: broadcast::Sender<ServerMsg>,
    mut bus_rx: broadcast::Receiver<ServerMsg>,
    active_nicks: SharedState,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let join_line = match ws_reader.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(_)) => return Err(anyhow::anyhow!("primeira mensagem deve ser texto JSON")),
        Some(Err(err)) => return Err(anyhow::anyhow!("erro de leitura websocket: {err}")),
        None => return Err(anyhow::anyhow!("conexao fechada antes do join")),
    };

    let nick = match serde_json::from_str::<ClientMsg>(&join_line)? {
        ClientMsg::Join { nick } => nick,
        _ => return Err(anyhow::anyhow!("primeira mensagem deve ser join")),
    };

    // Nick uniqueness check — drop lock before any await
    let nick_taken = {
        let mut nicks = active_nicks.lock().unwrap();
        if nicks.contains(&nick) {
            true
        } else {
            nicks.insert(nick.clone());
            false
        }
    };
    if nick_taken {
        send_msg(&mut ws_writer, &ServerMsg::JoinError {
            reason: format!("Nick '{}' já está em uso. Escolha outro.", nick),
        }).await?;
        return Ok(());
    }

    let mut inventory = persistence::load_inventory(&nick)
        .unwrap_or_else(initial_inventory);
    let character = initial_character();
    let recipes = all_recipes();

    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character }).await?;
    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: recipes.clone() }).await?;

    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} entrou no jogo"),
    });
    broadcast_players(&bus_tx, &active_nicks);

    loop {
        tokio::select! {
            incoming = ws_reader.next() => {
                let Some(incoming) = incoming else { break };

                match incoming {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientMsg>(&text) {
                            Ok(ClientMsg::Chat { text }) => {
                                let _ = bus_tx.send(ServerMsg::Chat {
                                    line: ChatLine { from: nick.clone(), text },
                                });
                            }
                            Ok(ClientMsg::Craft { recipe_id }) => {
                                let result = if let Some(recipe) = recipes.iter().find(|r| r.id == recipe_id) {
                                    if can_craft(&inventory, recipe) {
                                        apply_craft(&mut inventory, recipe);
                                        persistence::save_inventory(&nick, &inventory);
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                            items: inventory.clone(),
                                        }).await?;
                                        ServerMsg::CraftResult {
                                            success: true,
                                            message: format!("{} craftado com sucesso!", recipe.name),
                                        }
                                    } else {
                                        ServerMsg::CraftResult {
                                            success: false,
                                            message: "Ingredientes insuficientes.".to_string(),
                                        }
                                    }
                                } else {
                                    ServerMsg::CraftResult {
                                        success: false,
                                        message: "Receita desconhecida.".to_string(),
                                    }
                                };
                                send_msg(&mut ws_writer, &result).await?;
                            }
                            Ok(ClientMsg::Gather { action_id }) => {
                                let gather_actions = all_gather_actions();
                                let result = if let Some(action) = gather_actions.iter().find(|a| a.id == action_id) {
                                    let yielded = apply_gather(&mut inventory, action);
                                    persistence::save_inventory(&nick, &inventory);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                        items: inventory.clone(),
                                    }).await?;
                                    let items_desc = yielded.iter()
                                        .map(|i| format!("{} x{}", i.name, i.qty))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    ServerMsg::GatherResult {
                                        message: format!("Voce coletou: {items_desc}"),
                                        items: yielded,
                                    }
                                } else {
                                    ServerMsg::GatherResult {
                                        message: "Acao de coleta desconhecida.".to_string(),
                                        items: vec![],
                                    }
                                };
                                send_msg(&mut ws_writer, &result).await?;
                            }
                            Ok(ClientMsg::Join { .. }) => {}
                            Err(err) => {
                                eprintln!("mensagem invalida de {nick} ({peer_addr}): {err}");
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {}
                    Err(err) => {
                        let _ = bus_tx.send(ServerMsg::System {
                            text: format!("erro de websocket para {nick}: {err}"),
                        });
                        break;
                    }
                }
            }
            msg = bus_rx.recv() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        let warn = ServerMsg::System {
                            text: format!("aviso: voce perdeu {skipped} mensagens"),
                        };
                        send_msg(&mut ws_writer, &warn).await?;
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                send_msg(&mut ws_writer, &msg).await?;
            }
        }
    }

    active_nicks.lock().unwrap().remove(&nick);
    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} saiu do jogo"),
    });
    broadcast_players(&bus_tx, &active_nicks);

    Ok(())
}
