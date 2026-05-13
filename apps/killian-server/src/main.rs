mod craft;
mod gather;
mod items;
mod market;
mod mobs;
mod persistence;
mod quests;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use craft::{apply_craft, can_craft, craft_quality, recipes_for_player};
use gather::{all_gather_actions, apply_gather};
use items::{item_craft_power, item_gather_power, item_hp_restore, make_item};
use mobs::{all_mobs, apply_combat, dungeon_prereqs};
use market::Market;
use quests::{accept_quest, build_quest_list, on_items_gathered, on_mob_killed, turn_in_quest};
use futures_util::{SinkExt, StreamExt};
use killian_protocol::{
    CharacterData, ChatLine, ClientMsg, InventoryItem, ItemType, PlayerInfo,
    Rarity, ServerMsg, StatType,
};
use persistence::{
    alloc_stat, check_level_up, check_profession_level_up, default_character_save,
    prof_xp_for_level, save_all, xp_for_level, CharacterSave,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type WsWriter = futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>;
type SharedState = Arc<Mutex<HashMap<String, String>>>;
type SharedMarket = Arc<Mutex<Market>>;

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
    let shared_market: SharedMarket = Arc::new(Mutex::new(Market::load()));

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let bus_tx = bus_tx.clone();
        let bus_rx = bus_tx.subscribe();
        let active_players = active_players.clone();
        let shared_market = shared_market.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, peer_addr, bus_tx, bus_rx, active_players, shared_market).await {
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
    let quests = build_quest_list(&cs.active_quests);
    CharacterData {
        class_name: profession_display(&cs.profession),
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
        race: cs.race.clone(),
        profession: cs.profession.clone(),
        profession_level: cs.profession_level,
        profession_xp: cs.profession_xp,
        profession_xp_next: prof_xp_for_level(cs.profession_level),
        quests,
    }
}

fn profession_display(prof: &str) -> String {
    match prof {
        "ferreiro"   => "Ferreiro",
        "lenhador"   => "Lenhador",
        "minerador"  => "Minerador",
        "alquimista" => "Alquimista",
        _            => "Aventureiro",
    }.to_string()
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
    shared_market: SharedMarket,
) -> anyhow::Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    // First message must be Join
    let join_line = match ws_reader.next().await {
        Some(Ok(Message::Text(text))) => text.to_string(),
        Some(Ok(_)) => return Err(anyhow::anyhow!("primeira mensagem deve ser texto")),
        Some(Err(e)) => return Err(anyhow::anyhow!("erro ws: {e}")),
        None => return Err(anyhow::anyhow!("conexao fechada antes do join")),
    };

    let (nick, password) = match serde_json::from_str::<ClientMsg>(&join_line)? {
        ClientMsg::Join { nick, password } => (nick, password),
        _ => return Err(anyhow::anyhow!("primeira mensagem deve ser join")),
    };

    let password_hash = persistence::hash_password(&password);
    if let Some(player) = persistence::load_player(&nick) {
        if player.password_hash != password_hash {
            send_msg(&mut ws_writer, &ServerMsg::JoinError { reason: "Senha incorreta.".to_string() }).await?;
            return Ok(());
        }
    }

    let nick_taken = {
        let mut map = active_players.lock().unwrap();
        if map.contains_key(&nick) { true } else { map.insert(nick.clone(), "vila".to_string()); false }
    };
    if nick_taken {
        send_msg(&mut ws_writer, &ServerMsg::JoinError {
            reason: format!("Nick '{}' ja esta em uso.", nick),
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

    // Check if character needs creation (no race set)
    if char_save.race.is_empty() {
        send_msg(&mut ws_writer, &ServerMsg::NeedCharacterCreation).await?;

        // Wait for CreateCharacter message
        loop {
            match ws_reader.next().await {
                Some(Ok(Message::Text(text))) => {
                    match serde_json::from_str::<ClientMsg>(&text) {
                        Ok(ClientMsg::CreateCharacter { race, profession }) => {
                            let (sb, db, ab, vb) = race.stat_bonuses();
                            char_save.race = format!("{:?}", race).to_lowercase();
                            char_save.profession = format!("{:?}", profession).to_lowercase();
                            char_save.profession_level = 1;
                            char_save.profession_xp = 0;
                            // Apply race bonuses
                            char_save.str_stat = (char_save.str_stat as i32 + sb).max(1) as u32;
                            char_save.def_stat = (char_save.def_stat as i32 + db).max(1) as u32;
                            char_save.agi_stat = (char_save.agi_stat as i32 + ab).max(1) as u32;
                            char_save.vit_stat = (char_save.vit_stat as i32 + vb).max(1) as u32;
                            char_save.max_hp = 80 + (char_save.vit_stat * 10) as i32;
                            char_save.hp = char_save.max_hp;
                            persistence::save_all(&nick, &inventory, &char_save);
                            send_msg(&mut ws_writer, &ServerMsg::CharacterCreationOk).await?;
                            break;
                        }
                        _ => {}
                    }
                }
                Some(Ok(Message::Close(_))) | None => {
                    active_players.lock().unwrap().remove(&nick);
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    // Dungeon kill tracking for this session (mob_id -> count)
    let mut dungeon_kills: HashMap<String, u32> = HashMap::new();

    let recipes = recipes_for_player(&char_save);

    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: recipes.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::EquipUpdate { equipped: char_save.equipped.clone() }).await?;
    send_msg(&mut ws_writer, &ServerMsg::QuestUpdate { quests: build_quest_list(&char_save.active_quests) }).await?;
    let market_listings = shared_market.lock().unwrap().to_protocol();
    send_msg(&mut ws_writer, &ServerMsg::MarketUpdate { listings: market_listings }).await?;

    let _ = bus_tx.send(ServerMsg::System { text: format!("{nick} entrou no jogo") });
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
                                active_players.lock().unwrap().insert(nick.clone(), zone_id.clone());
                                // Reset dungeon kills on zone change
                                if zone_id == "toca_das_sombras" {
                                    dungeon_kills.clear();
                                }
                                broadcast_players(&bus_tx, &active_players);
                            }

                            Ok(ClientMsg::Equip { item_name }) => {
                                if let Some(item) = inventory.iter().find(|i| i.name == item_name) {
                                    let equippable = matches!(
                                        item.item_type,
                                        ItemType::Weapon | ItemType::Armor | ItemType::Ring | ItemType::Tool
                                    );
                                    if !equippable {
                                        send_msg(&mut ws_writer, &ServerMsg::System { text: "Este item nao pode ser equipado.".into() }).await?;
                                    } else if char_save.equipped.contains(&item_name) {
                                        send_msg(&mut ws_writer, &ServerMsg::System { text: "Item ja esta equipado.".into() }).await?;
                                    } else {
                                        char_save.equipped.push(item_name.clone());
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::EquipUpdate { equipped: char_save.equipped.clone() }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::System { text: format!("{item_name} equipado!") }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::Unequip { item_name }) => {
                                if let Some(pos) = char_save.equipped.iter().position(|e| e == &item_name) {
                                    char_save.equipped.remove(pos);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::EquipUpdate { equipped: char_save.equipped.clone() }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: format!("{item_name} desequipado.") }).await?;
                                }
                            }

                            Ok(ClientMsg::Craft { recipe_id }) => {
                                let recipes = recipes_for_player(&char_save);
                                let result = if let Some(recipe) = recipes.iter().find(|r| r.id == recipe_id) {
                                    if can_craft(&inventory, recipe) {
                                        let craft_power = item_craft_power(&inventory, &char_save.equipped);
                                        let quality = craft_quality(char_save.profession_level, craft_power);
                                        let crafted = apply_craft(&mut inventory, recipe, quality.clone());
                                        // Profession XP
                                        char_save.profession_xp += 10;
                                        if check_profession_level_up(&mut char_save) {
                                            let _ = bus_tx.send(ServerMsg::System {
                                                text: format!("{nick} subiu para Nivel {} de {}!", char_save.profession_level, profession_display(&char_save.profession)),
                                            });
                                            // Refresh recipes on level-up
                                            let new_recipes = recipes_for_player(&char_save);
                                            send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: new_recipes }).await?;
                                        }
                                        // Announce epic crafts globally
                                        if crafted.rarity >= Rarity::Epic {
                                            let qual = rarity_name(&crafted.rarity);
                                            let _ = bus_tx.send(ServerMsg::GlobalAnnouncement {
                                                text: format!("[CRAFTING] {} criou um item {}! {}", nick, qual, crafted.name),
                                            });
                                        }
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                        ServerMsg::CraftResult { success: true, message: format!("{} ({}) craftado!", recipe.name, rarity_name(&quality)) }
                                    } else {
                                        ServerMsg::CraftResult { success: false, message: "Ingredientes insuficientes.".into() }
                                    }
                                } else {
                                    ServerMsg::CraftResult { success: false, message: "Receita desconhecida.".into() }
                                };
                                send_msg(&mut ws_writer, &result).await?;
                            }

                            Ok(ClientMsg::Gather { action_id }) => {
                                let gather_actions = all_gather_actions();
                                if let Some(action) = gather_actions.iter().find(|a| a.id == action_id) {
                                    let gather_power = item_gather_power(&inventory, &char_save.equipped);
                                    let result = apply_gather(&mut inventory, action, gather_power);
                                    if let Some(reason) = result.blocked {
                                        send_msg(&mut ws_writer, &ServerMsg::GatherResult {
                                            message: reason.to_string(),
                                            items: vec![],
                                        }).await?;
                                    } else {
                                        // Profession XP for matching profession
                                        if action.profession == char_save.profession {
                                            char_save.profession_xp += 5;
                                            if check_profession_level_up(&mut char_save) {
                                                let _ = bus_tx.send(ServerMsg::System {
                                                    text: format!("{nick} subiu para Nivel {} de {}!", char_save.profession_level, profession_display(&char_save.profession)),
                                                });
                                                let new_recipes = recipes_for_player(&char_save);
                                                send_msg(&mut ws_writer, &ServerMsg::RecipesUpdate { recipes: new_recipes }).await?;
                                            }
                                        }
                                        // Quest progress
                                        let qty_gathered: u32 = result.items.iter().map(|i| i.qty).sum();
                                        on_items_gathered(&mut char_save.active_quests, action.yield_item, qty_gathered);
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::QuestUpdate { quests: build_quest_list(&char_save.active_quests) }).await?;
                                        let items_desc = result.items.iter()
                                            .map(|i| format!("{} x{}", i.name, i.qty))
                                            .collect::<Vec<_>>()
                                            .join(", ");
                                        send_msg(&mut ws_writer, &ServerMsg::GatherResult {
                                            message: format!("{} ({}): {items_desc}", action.name, action.location),
                                            items: result.items,
                                        }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::Attack { mob_id }) => {
                                // Dungeon boss prerequisite check
                                if let Some(prereqs) = dungeon_prereqs(&mob_id) {
                                    let mut missing = vec![];
                                    for (req_mob, req_count) in &prereqs {
                                        let have = dungeon_kills.get(*req_mob).copied().unwrap_or(0);
                                        if have < *req_count {
                                            missing.push(format!("{} ({}/{})", req_mob, have, req_count));
                                        }
                                    }
                                    if !missing.is_empty() {
                                        send_msg(&mut ws_writer, &ServerMsg::CombatResult {
                                            won: false,
                                            message: format!("O boss ainda esta protegido! Mate primeiro: {}", missing.join(", ")),
                                            loot: vec![],
                                        }).await?;
                                        continue;
                                    }
                                }

                                if let Some(mob) = all_mobs().iter().find(|m| m.id == mob_id) {
                                    let (_, eff_def_bonus, _, _) = equipment_bonuses(&inventory, &char_save.equipped);
                                    let effective_def = char_save.def_stat + eff_def_bonus;
                                    let outcome = apply_combat(&mut inventory, mob, &mut char_save, effective_def);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;

                                    if outcome.died {
                                        send_msg(&mut ws_writer, &ServerMsg::CombatResult {
                                            won: false,
                                            message: format!("Voce foi derrotado por {}! Perdeu 10% do ouro.", mob.name),
                                            loot: vec![],
                                        }).await?;
                                    } else {
                                        // Track dungeon kills
                                        *dungeon_kills.entry(mob_id.clone()).or_default() += 1;
                                        // Quest progress
                                        on_mob_killed(&mut char_save.active_quests, &mob_id);

                                        let desc = if outcome.loot.is_empty() {
                                            "Nenhum item.".into()
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
                                            loot: outcome.loot.clone(),
                                        }).await?;

                                        // Epic drop announcement
                                        if outcome.has_epic_drop {
                                            if let Some(ref item_name) = outcome.epic_item_name {
                                                let rarity = outcome.loot.iter()
                                                    .find(|i| &i.name == item_name)
                                                    .map(|i| rarity_name(&i.rarity))
                                                    .unwrap_or("Epico");
                                                let _ = bus_tx.send(ServerMsg::GlobalAnnouncement {
                                                    text: format!("[LOOT] {} dropou um item {}! {}", nick, rarity, item_name),
                                                });
                                            }
                                        }

                                        if check_level_up(&mut char_save) {
                                            save_all(&nick, &inventory, &char_save);
                                            let _ = bus_tx.send(ServerMsg::System {
                                                text: format!("{nick} subiu para nivel {}!", char_save.level),
                                            });
                                        }
                                        send_msg(&mut ws_writer, &ServerMsg::QuestUpdate { quests: build_quest_list(&char_save.active_quests) }).await?;
                                    }
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                }
                            }

                            Ok(ClientMsg::UseItem { item_name }) => {
                                let hp_gain = item_hp_restore(&item_name);
                                if hp_gain == 0 {
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: "Item nao pode ser usado.".into() }).await?;
                                } else if let Some(pos) = inventory.iter().position(|i| i.name == item_name) {
                                    inventory[pos].qty -= 1;
                                    if inventory[pos].qty == 0 { inventory.remove(pos); }
                                    char_save.hp = (char_save.hp + hp_gain as i32).min(char_save.max_hp);
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: format!("{item_name} usada! +{hp_gain} HP") }).await?;
                                }
                            }

                            Ok(ClientMsg::AllocStat { stat }) => {
                                if alloc_stat(&mut char_save, &stat) {
                                    save_all(&nick, &inventory, &char_save);
                                    let stat_name = match &stat {
                                        StatType::Str => "FOR", StatType::Def => "DEF",
                                        StatType::Agi => "AGI", StatType::Vit => "VIT",
                                    };
                                    send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: format!("{stat_name} aumentado!") }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: "Sem pontos de atributo.".into() }).await?;
                                }
                            }

                            Ok(ClientMsg::AcceptQuest { quest_id }) => {
                                if accept_quest(&mut char_save.active_quests, &quest_id) {
                                    save_all(&nick, &inventory, &char_save);
                                    send_msg(&mut ws_writer, &ServerMsg::QuestUpdate { quests: build_quest_list(&char_save.active_quests) }).await?;
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: format!("Missao aceita!") }).await?;
                                } else {
                                    send_msg(&mut ws_writer, &ServerMsg::System { text: "Nao e possivel aceitar esta missao.".into() }).await?;
                                }
                            }

                            Ok(ClientMsg::TurnInQuest { quest_id }) => {
                                match turn_in_quest(&mut char_save.active_quests, &quest_id) {
                                    Some((xp, gold)) => {
                                        char_save.xp += xp;
                                        char_save.gold += gold;
                                        if check_level_up(&mut char_save) {
                                            let _ = bus_tx.send(ServerMsg::System {
                                                text: format!("{nick} subiu para nivel {}!", char_save.level),
                                            });
                                        }
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::QuestUpdate { quests: build_quest_list(&char_save.active_quests) }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::System { text: format!("Missao concluida! +{xp}xp +{gold}g") }).await?;
                                    }
                                    None => {
                                        send_msg(&mut ws_writer, &ServerMsg::System { text: "Missao ainda nao concluida.".into() }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::ListItem { item_name, qty, price }) => {
                                // Find item in inventory
                                let pos = inventory.iter().position(|i| i.name == item_name);
                                let result = if let Some(pos) = pos {
                                    if inventory[pos].qty < qty {
                                        Err("Quantidade insuficiente no inventário.")
                                    } else {
                                        let mut listed_item = inventory[pos].clone();
                                        listed_item.qty = qty;
                                        inventory[pos].qty -= qty;
                                        if inventory[pos].qty == 0 { inventory.remove(pos); }
                                        let mut mkt = shared_market.lock().unwrap();
                                        match mkt.list_item(&nick, listed_item, price) {
                                            Ok(id) => Ok(id),
                                            Err(e) => {
                                                // Return item on failure
                                                Err(e)
                                            }
                                        }
                                    }
                                } else {
                                    Err("Item não encontrado no inventário.")
                                };
                                match result {
                                    Ok(_id) => {
                                        let listings = shared_market.lock().unwrap().to_protocol();
                                        save_all(&nick, &inventory, &char_save);
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                        let _ = bus_tx.send(ServerMsg::MarketUpdate { listings });
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: true, message: format!("{} x{} listado por {}g.", item_name, qty, price) }).await?;
                                    }
                                    Err(reason) => {
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: false, message: reason.to_string() }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::BuyItem { listing_id }) => {
                                let result = shared_market.lock().unwrap().buy_item(&nick, listing_id, char_save.gold);
                                match result {
                                    Ok(entry) => {
                                        char_save.gold -= entry.price;
                                        // Credit seller (update their save file)
                                        if let Some(mut seller_data) = persistence::load_player(&entry.seller) {
                                            seller_data.character.gold += entry.price;
                                            persistence::save_player(&entry.seller, &seller_data);
                                            // Notify seller if online
                                            let _ = bus_tx.send(ServerMsg::System {
                                                text: format!("[Mercado] {} comprou {} por {}g — ouro creditado!", nick, entry.item.name, entry.price),
                                            });
                                        }
                                        // Add item to buyer
                                        if let Some(existing) = inventory.iter_mut().find(|i| i.name == entry.item.name) {
                                            existing.qty += entry.item.qty;
                                        } else {
                                            inventory.push(entry.item.clone());
                                        }
                                        save_all(&nick, &inventory, &char_save);
                                        let listings = shared_market.lock().unwrap().to_protocol();
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                        send_msg(&mut ws_writer, &ServerMsg::CharacterUpdate { character: char_save_to_data(&char_save, &inventory) }).await?;
                                        let _ = bus_tx.send(ServerMsg::MarketUpdate { listings });
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: true, message: format!("Comprou {} x{} por {}g!", entry.item.name, entry.item.qty, entry.price) }).await?;
                                    }
                                    Err(reason) => {
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: false, message: reason.to_string() }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::CancelListing { listing_id }) => {
                                let result = shared_market.lock().unwrap().cancel_listing(&nick, listing_id);
                                match result {
                                    Ok(entry) => {
                                        // Return item to inventory
                                        if let Some(existing) = inventory.iter_mut().find(|i| i.name == entry.item.name) {
                                            existing.qty += entry.item.qty;
                                        } else {
                                            inventory.push(entry.item.clone());
                                        }
                                        save_all(&nick, &inventory, &char_save);
                                        let listings = shared_market.lock().unwrap().to_protocol();
                                        send_msg(&mut ws_writer, &ServerMsg::InventoryUpdate { items: inventory.clone() }).await?;
                                        let _ = bus_tx.send(ServerMsg::MarketUpdate { listings });
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: true, message: format!("Listagem cancelada. {} devolvido ao inventário.", entry.item.name) }).await?;
                                    }
                                    Err(reason) => {
                                        send_msg(&mut ws_writer, &ServerMsg::MarketResult { success: false, message: reason.to_string() }).await?;
                                    }
                                }
                            }

                            Ok(ClientMsg::Join { .. }) | Ok(ClientMsg::CreateCharacter { .. }) => {}

                            Err(err) => {
                                eprintln!("mensagem invalida de {nick} ({peer_addr}): {err}");
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {}
                    Err(err) => {
                        let _ = bus_tx.send(ServerMsg::System { text: format!("erro ws {nick}: {err}") });
                        break;
                    }
                }
            }

            msg = bus_rx.recv() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        send_msg(&mut ws_writer, &ServerMsg::System {
                            text: format!("aviso: voce perdeu {skipped} mensagens"),
                        }).await?;
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                send_msg(&mut ws_writer, &msg).await?;
            }
        }
    }

    active_players.lock().unwrap().remove(&nick);
    let _ = bus_tx.send(ServerMsg::System { text: format!("{nick} saiu do jogo") });
    broadcast_players(&bus_tx, &active_players);

    Ok(())
}

fn rarity_name(r: &Rarity) -> &'static str {
    match r {
        Rarity::Ruim        => "Ruim",
        Rarity::Common      => "Comum",
        Rarity::Uncommon    => "Incomum",
        Rarity::Rare        => "Raro",
        Rarity::Epic        => "Epico",
        Rarity::RefinedEpic => "Epico Refinado",
        Rarity::Legendary   => "Lendario",
        Rarity::Broken      => "Quebrado",
    }
}
