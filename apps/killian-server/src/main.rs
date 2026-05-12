mod craft;
mod gather;
mod items;
mod mobs;
mod persistence;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use craft::{all_recipes, apply_craft, can_craft};
use gather::{all_gather_actions, apply_gather};
use items::{item_hp_restore, make_item};
use mobs::{all_mobs, apply_combat};
use futures_util::{SinkExt, StreamExt};
use killian_protocol::{CharacterData, ChatLine, ClientMsg, InventoryItem, ItemType, PlayerInfo, ServerMsg, StatType};
use persistence::{alloc_stat, check_level_up, default_character_save, save_all, xp_for_level, CharacterSave};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type WsWriter = futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>;
// Maps nick -> current zone
type SharedState = Arc<Mutex<HashMap<String, String>>>;

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
    let active_players: SharedState = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let bus_tx = bus_tx.clone();
        let bus_rx = bus_tx.subscribe();
        let active_players = active_players.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, peer_addr, bus_tx, bus_rx, active_players).await {
                eprintln!("erro cliente {}: {err}", peer_addr);
            }
        });
    }
}

fn initial_inventory() -> Vec<InventoryItem> {
    vec![
        make_item("Pocao Pequena", 3),
        make_item("Espada Curta", 1),
        make_item("Madeira", 12),
        make_item("Pedra", 6),
    ]
}

fn equipment_bonuses(inventory: &[InventoryItem], equipped: &[String]) -> (u32, u32, u32, u32) {
    equipped.iter().fold((0u32, 0u32, 0u32, 0u32), |acc, name| {
        if let Some(item) = inventory.iter().find(|i| i.name == *name) {
            (acc.0 + item.str_bonus, acc.1 + item.def_bonus, acc.2 + item.agi_bonus, acc.3 + item.vit_bonus)
        } else {
            acc
        }
    })
}

fn char_save_to_data(cs: &CharacterSave, inventory: &[InventoryItem]) -> CharacterData {
    let (sb, db, ab, vb) = equipment_bonuses(inventory, &cs.equipped);
    CharacterData {
        class_name: "Aventureiro".to_string(),
        level: cs.level,
        hp: cs.hp,
        max_hp: cs.max_hp,
        mp: 35,
        max_mp: 35,
        gold: cs.gold,
        xp: cs.xp,
        xp_next: xp_for_level(cs.level),
        str_stat: cs.str_stat + sb,
        def_stat: cs.def_stat + db,
        agi_stat: cs.agi_stat + ab,
        vit_stat: cs.vit_stat + vb,
        stat_points: cs.stat_points,
    }
}

fn broadcast_players(bus_tx: &broadcast::Sender<ServerMsg>, active_players: &SharedState) {
    let map = active_players.lock().unwrap();
    let mut players: Vec<PlayerInfo> = map.iter()
        .map(|(nick, zone)| PlayerInfo { nick: nick.clone(), zone: zone.clone() })
        .collect();
    players.sort_by(|a, b| a.nick.cmp(&b.nick));
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
    active_players: SharedState,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let join_line = match ws_reader.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(_)) => return Err(anyhow::anyhow!("primeira mensagem deve ser texto JSON")),
        Some(Err(err)) => return Err(anyhow::anyhow!("erro de leitura websocket: {err}")),
        None => return Err(anyhow::anyhow!("conexao fechada antes do join")),
    };

    let (nick, password) = match serde_json::from_str::<ClientMsg>(&join_line)? {
        ClientMsg::Join { nick, password } => (nick, password),
        _ => return Err(anyhow::anyhow!("primeira mensagem deve ser join")),
    };

    let password_hash = persistence::hash_password(&password);
    match persistence::load_player(&nick) {
        Some(player) if player.password_hash != password_hash => {
            send_msg(&mut ws_writer, &ServerMsg::JoinError {
                reason: "Senha incorreta.".to_string(),
            }).await?;
            return Ok(());
        }
        _ => {}
    }

    let nick_taken = {
        let mut map = active_players.lock().unwrap();
        if map.contains_key(&nick) {
            true
        } else {
            map.insert(nick.clone(), "vila".to_string());
            false
        }
    };
    if nick_taken {
        send_msg(&mut ws_writer, &ServerMsg::JoinError {
            reason: format!("Nick '{}' já está em uso. Escolha outro.", nick),
        }).await?;
        return Ok(());
    }

    let is_new = persistence::load_player(&nick).is_none();
    let player_data = persistence::load_player(&nick).unwrap_or_else(|| persistence::PlayerData {
        password_hash: password_hash.clone(),
        inventory: initial_inventory(),
        character: default_character_save(),
    });

    let mut inventory = player_data.inventory.clone();
    let mut char_save = player_data.character.clone();

    if is_new {
        persistence::save_player(&nick, &player_data);
    }

    let recipes = all_recipes();

    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: recipes.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::EquipUpdate { equipped: char_save.equipped.clone() }).await?;

    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} entrou no jogo"),
    });
    broadcast_players(&bus_tx, &active_players);

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
                            Ok(ClientMsg::Travel { zone_id }) => {
                                active_players.lock().unwrap().insert(nick.clone(), zone_id);
                                broadcast_players(&bus_tx, &active_players);
                            }
                            Ok(ClientMsg::Equip { item_name }) => {
                                if let Some(item) = inventory.iter().find(|i| i.name == item_name) {
                                    let equippable = matches!(
                                        item.item_type,
                                        ItemType::Weapon | ItemType::Armor | ItemType::Ring
                                    );
                                    if !equippable {
                                        send_msg(&mut ws_writer, &ServerMsg::System {
                                            text: "Este item nao pode ser equipado.".to_string(),
                                        }).await?;
                                    } else if char_save.equipped.contains(&item_name) {
                                        send_msg(&mut ws_writer, &ServerMsg::System {
                                            text: "Item ja esta equipado.".to_string(),
                                        }).await?;
                                    } else {
                                        char_save.equipped.push(item_name.clone());
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::EquipUpdate {
                                            equipped: char_save.equipped.clone(),
                                        }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate {
                                            character: char_save_to_data(&char_save, &inventory),
                                        }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::System {
                                            text: format!("{item_name} equipado!"),
                                        }).await?;
                                    }
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: "Item nao encontrado no inventario.".to_string(),
                                    }).await?;
                                }
                            }
                            Ok(ClientMsg::Unequip { item_name }) => {
                                if let Some(pos) = char_save.equipped.iter().position(|e| e == &item_name) {
                                    char_save.equipped.remove(pos);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::EquipUpdate {
                                        equipped: char_save.equipped.clone(),
                                    }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate {
                                        character: char_save_to_data(&char_save, &inventory),
                                    }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: format!("{item_name} desequipado."),
                                    }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: "Item nao esta equipado.".to_string(),
                                    }).await?;
                                }
                            }
                            Ok(ClientMsg::Craft { recipe_id }) => {
                                let result = if let Some(recipe) = recipes.iter().find(|r| r.id == recipe_id) {
                                    if can_craft(&inventory, recipe) {
                                        apply_craft(&mut inventory, recipe);
                                        save_all(&nick, &inventory, &char_save);
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
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                        items: inventory.clone(),
                                    }).await?;
                                    let items_desc = yielded.iter()
                                        .map(|i| format!("{} x{}", i.name, i.qty))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    ServerMsg::GatherResult {
                                        message: format!("{} ({}): {items_desc}", action.name, action.location),
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
                            Ok(ClientMsg::Attack { mob_id }) => {
                                if let Some(mob) = all_mobs().iter().find(|m| m.id == mob_id) {
                                    let (_, eff_def_bonus, _, _) = equipment_bonuses(&inventory, &char_save.equipped);
                                    let effective_def = char_save.def_stat + eff_def_bonus;
                                    let outcome = apply_combat(&mut inventory, mob, &mut char_save, effective_def);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                        items: inventory.clone(),
                                    }).await?;

                                    if outcome.died {
                                        send_msg(&mut ws_writer, &ServerMsg::CombatResult {
                                            won: false,
                                            message: format!(
                                                "Voce foi derrotado por {}! Perdeu 10% do ouro.",
                                                mob.name
                                            ),
                                            loot: vec![],
                                        }).await?;
                                    } else {
                                        let desc = if outcome.loot.is_empty() {
                                            "Nenhum item.".to_string()
                                        } else {
                                            outcome.loot.iter()
                                                .map(|i| format!("{} x{}", i.name, i.qty))
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        };
                                        send_msg(&mut ws_writer, &ServerMsg::CombatResult {
                                            won: true,
                                            message: format!(
                                                "{} derrotado! +{}xp +{}g -{} HP  {}",
                                                mob.name, outcome.xp_gained, outcome.gold_gained,
                                                outcome.damage_taken, desc
                                            ),
                                            loot: outcome.loot,
                                        }).await?;

                                        if check_level_up(&mut char_save) {
                                            save_all(&nick, &inventory, &char_save);
                                            let _ = bus_tx.send(ServerMsg::System {
                                                text: format!("{nick} subiu para nivel {}!", char_save.level),
                                            });
                                        }
                                    }

                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate {
                                        character: char_save_to_data(&char_save, &inventory),
                                    }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::CombatResult {
                                        won: false,
                                        message: "Inimigo desconhecido.".to_string(),
                                        loot: vec![],
                                    }).await?;
                                }
                            }
                            Ok(ClientMsg::UseItem { item_name }) => {
                                let hp_gain = item_hp_restore(&item_name);
                                if hp_gain == 0 {
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: "Item nao pode ser usado.".to_string(),
                                    }).await?;
                                } else if let Some(pos) = inventory.iter().position(|i| i.name == item_name) {
                                    inventory[pos].qty -= 1;
                                    if inventory[pos].qty == 0 {
                                        inventory.remove(pos);
                                    }
                                    char_save.hp = (char_save.hp + hp_gain as i32).min(char_save.max_hp);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate {
                                        items: inventory.clone(),
                                    }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate {
                                        character: char_save_to_data(&char_save, &inventory),
                                    }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: format!("{item_name} usada! +{hp_gain} HP"),
                                    }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: "Item nao encontrado no inventario.".to_string(),
                                    }).await?;
                                }
                            }
                            Ok(ClientMsg::AllocStat { stat }) => {
                                if alloc_stat(&mut char_save, &stat) {
                                    save_all(&nick, &inventory, &char_save);
                                    let stat_name = match &stat {
                                        StatType::Str => "FOR",
                                        StatType::Def => "DEF",
                                        StatType::Agi => "AGI",
                                        StatType::Vit => "VIT",
                                    };
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate {
                                        character: char_save_to_data(&char_save, &inventory),
                                    }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: format!("{stat_name} aumentado!"),
                                    }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System {
                                        text: "Sem pontos de atributo.".to_string(),
                                    }).await?;
                                }
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

    active_players.lock().unwrap().remove(&nick);
    let _ = bus_tx.send(ServerMsg::System {
        text: format!("{nick} saiu do jogo"),
    });
    broadcast_players(&bus_tx, &active_players);

    Ok(())
}
