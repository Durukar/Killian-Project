use killian_protocol::{InventoryItem, MobType, Rarity};
use rand::Rng;

use crate::items::make_item;
use crate::persistence::CharacterSave;

pub struct LootEntry {
    pub item: &'static str,
    pub min_qty: u32,
    pub max_qty: u32,
    pub chance: f32,
    pub min_rarity: Rarity,
}

pub struct Mob {
    pub id: &'static str,
    pub name: &'static str,
    pub zone: &'static str,
    pub level: u32,
    pub hp: u32,
    pub atk: u32,
    pub def: u32,
    pub xp_reward: u32,
    pub gold_reward: u32,
    pub mob_type: MobType,
    pub loot: &'static [LootEntry],
}

pub fn all_mobs() -> &'static [Mob] {
    &[
        // FLORESTA
        Mob {
            id: "goblin", name: "Goblin", zone: "floresta",
            level: 1, hp: 30, atk: 5, def: 2, xp_reward: 20, gold_reward: 5,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Madeira",       min_qty: 1, max_qty: 3, chance: 0.8, min_rarity: Rarity::Common },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.3, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "lobo", name: "Lobo", zone: "floresta",
            level: 2, hp: 45, atk: 8, def: 3, xp_reward: 35, gold_reward: 8,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Madeira",       min_qty: 2, max_qty: 4, chance: 0.6, min_rarity: Rarity::Common },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "lobo_alfa", name: "[ELITE] Lobo Alfa", zone: "floresta",
            level: 3, hp: 110, atk: 14, def: 6, xp_reward: 90, gold_reward: 30,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Madeira",         min_qty: 3, max_qty: 6, chance: 0.9, min_rarity: Rarity::Common },
                LootEntry { item: "Luvas de Couro",  min_qty: 1, max_qty: 1, chance: 0.20, min_rarity: Rarity::Uncommon },
            ],
        },
        // MINA
        Mob {
            id: "morcego", name: "Morcego", zone: "mina",
            level: 2, hp: 25, atk: 6, def: 1, xp_reward: 25, gold_reward: 4,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pedra", min_qty: 1, max_qty: 2, chance: 0.9, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "esqueleto", name: "Esqueleto", zone: "mina",
            level: 3, hp: 60, atk: 10, def: 5, xp_reward: 50, gold_reward: 12,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pedra",         min_qty: 2, max_qty: 4, chance: 0.7, min_rarity: Rarity::Common },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.4, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "esqueleto_guerreiro", name: "[ELITE] Esqueleto Guerreiro", zone: "mina",
            level: 4, hp: 130, atk: 16, def: 9, xp_reward: 110, gold_reward: 40,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Pedra",           min_qty: 3, max_qty: 6, chance: 0.8, min_rarity: Rarity::Common },
                LootEntry { item: "Espada Curta",    min_qty: 1, max_qty: 1, chance: 0.15, min_rarity: Rarity::Uncommon },
            ],
        },
        // PANTANO
        Mob {
            id: "sanguessuga", name: "Sanguessuga", zone: "pantano",
            level: 2, hp: 40, atk: 9, def: 2, xp_reward: 30, gold_reward: 6,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "basilisco", name: "Basilisco", zone: "pantano",
            level: 4, hp: 90, atk: 16, def: 7, xp_reward: 80, gold_reward: 18,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Media", min_qty: 1, max_qty: 1, chance: 0.4, min_rarity: Rarity::Common },
                LootEntry { item: "Pedra",       min_qty: 2, max_qty: 4, chance: 0.6, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "basilisco_anciao", name: "[ELITE RARO] Basilisco Anciao", zone: "pantano",
            level: 6, hp: 250, atk: 26, def: 14, xp_reward: 280, gold_reward: 100,
            mob_type: MobType::RareElite,
            loot: &[
                LootEntry { item: "Pocao Media",       min_qty: 2, max_qty: 3, chance: 1.0, min_rarity: Rarity::Common },
                LootEntry { item: "Anel do Pantano",   min_qty: 1, max_qty: 1, chance: 0.25, min_rarity: Rarity::Rare },
            ],
        },
        // CAVERNA
        Mob {
            id: "aranha", name: "Aranha", zone: "caverna",
            level: 2, hp: 35, atk: 8, def: 2, xp_reward: 28, gold_reward: 5,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pedra", min_qty: 1, max_qty: 2, chance: 0.8, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "golem", name: "Golem de Pedra", zone: "caverna",
            level: 5, hp: 120, atk: 18, def: 12, xp_reward: 110, gold_reward: 25,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Pedra",            min_qty: 3, max_qty: 6, chance: 0.9, min_rarity: Rarity::Common },
                LootEntry { item: "Botas de Combate", min_qty: 1, max_qty: 1, chance: 0.18, min_rarity: Rarity::Uncommon },
            ],
        },
        // CAMPOS
        Mob {
            id: "javali", name: "Javali", zone: "campos",
            level: 3, hp: 70, atk: 12, def: 4, xp_reward: 55, gold_reward: 10,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 2, chance: 0.6, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "bandido", name: "Bandido", zone: "campos",
            level: 4, hp: 80, atk: 14, def: 6, xp_reward: 70, gold_reward: 20,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5, min_rarity: Rarity::Common },
                LootEntry { item: "Madeira",       min_qty: 1, max_qty: 2, chance: 0.4, min_rarity: Rarity::Common },
            ],
        },
        // MONTANHA
        Mob {
            id: "urso", name: "Urso", zone: "montanha",
            level: 4, hp: 100, atk: 17, def: 8, xp_reward: 90, gold_reward: 15,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Media", min_qty: 1, max_qty: 1, chance: 0.5, min_rarity: Rarity::Common },
                LootEntry { item: "Madeira",     min_qty: 1, max_qty: 3, chance: 0.5, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "gigante", name: "Gigante das Montanhas", zone: "montanha",
            level: 6, hp: 160, atk: 22, def: 14, xp_reward: 150, gold_reward: 35,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Pocao Media",      min_qty: 1, max_qty: 2, chance: 0.6, min_rarity: Rarity::Common },
                LootEntry { item: "Coraca de Ferro",  min_qty: 1, max_qty: 1, chance: 0.15, min_rarity: Rarity::Uncommon },
            ],
        },
        // DESERTO
        Mob {
            id: "escorpiao", name: "Escorpiao", zone: "deserto",
            level: 3, hp: 55, atk: 11, def: 5, xp_reward: 48, gold_reward: 12,
            mob_type: MobType::Common,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5, min_rarity: Rarity::Common },
            ],
        },
        Mob {
            id: "drake", name: "Drake do Deserto", zone: "deserto",
            level: 6, hp: 150, atk: 24, def: 10, xp_reward: 160, gold_reward: 40,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Pocao Media",  min_qty: 1, max_qty: 2, chance: 0.7, min_rarity: Rarity::Common },
                LootEntry { item: "Espada Longa", min_qty: 1, max_qty: 1, chance: 0.12, min_rarity: Rarity::Uncommon },
            ],
        },
        // TOCA DAS SOMBRAS (dungeon)
        Mob {
            id: "guarda_sombra", name: "Guarda Sombra", zone: "toca_das_sombras",
            level: 7, hp: 180, atk: 28, def: 12, xp_reward: 200, gold_reward: 50,
            mob_type: MobType::Elite,
            loot: &[
                LootEntry { item: "Pocao Media",     min_qty: 1, max_qty: 1, chance: 0.6, min_rarity: Rarity::Common },
                LootEntry { item: "Coraca de Ferro", min_qty: 1, max_qty: 1, chance: 0.20, min_rarity: Rarity::Rare },
            ],
        },
        Mob {
            id: "arcanista_negro", name: "[ELITE RARO] Arcanista Negro", zone: "toca_das_sombras",
            level: 8, hp: 140, atk: 35, def: 8, xp_reward: 240, gold_reward: 60,
            mob_type: MobType::RareElite,
            loot: &[
                LootEntry { item: "Pocao Media",      min_qty: 1, max_qty: 2, chance: 0.7, min_rarity: Rarity::Common },
                LootEntry { item: "Manto das Sombras",min_qty: 1, max_qty: 1, chance: 0.18, min_rarity: Rarity::Rare },
            ],
        },
        Mob {
            id: "senhor_sombras", name: "Senhor das Sombras", zone: "toca_das_sombras",
            level: 10, hp: 350, atk: 42, def: 16, xp_reward: 500, gold_reward: 150,
            mob_type: MobType::Boss,
            loot: &[
                LootEntry { item: "Pocao Media",         min_qty: 2, max_qty: 3, chance: 1.0, min_rarity: Rarity::Common },
                LootEntry { item: "Amuleto Antigo",      min_qty: 1, max_qty: 1, chance: 0.40, min_rarity: Rarity::Epic },
                LootEntry { item: "Lanca das Sombras",   min_qty: 1, max_qty: 1, chance: 0.15, min_rarity: Rarity::RefinedEpic },
            ],
        },
    ]
}

// Dungeon boss prerequisite: kills needed per mob before boss is available
pub fn dungeon_prereqs(mob_id: &str) -> Option<Vec<(&'static str, u32)>> {
    match mob_id {
        "senhor_sombras" => Some(vec![
            ("guarda_sombra", 2),
            ("arcanista_negro", 1),
        ]),
        _ => None,
    }
}

// Returns damage multiplier based on player vs mob level difference
pub fn damage_multiplier(player_level: u32, mob_level: u32) -> i32 {
    let diff = mob_level as i32 - player_level as i32;
    if diff <= 0 { 1 }
    else if diff <= 2 { 2 }
    else if diff <= 4 { 3 }
    else { 10 }
}

pub fn max_rarity_for_mob_type(mob_type: &MobType) -> Rarity {
    match mob_type {
        MobType::Common    => Rarity::Uncommon,
        MobType::Elite     => Rarity::Rare,
        MobType::RareElite => Rarity::RefinedEpic,
        MobType::Boss      => Rarity::RefinedEpic,
    }
}

pub struct CombatOutcome {
    pub loot: Vec<InventoryItem>,
    pub xp_gained: u32,
    pub gold_gained: u32,
    pub damage_taken: i32,
    pub died: bool,
    pub has_epic_drop: bool,
    pub epic_item_name: Option<String>,
}

pub fn apply_combat(
    inventory: &mut Vec<InventoryItem>,
    mob: &Mob,
    char_save: &mut CharacterSave,
    effective_def: u32,
) -> CombatOutcome {
    let mut rng = rand::rng();
    let mut gained = Vec::new();
    let max_rarity = max_rarity_for_mob_type(&mob.mob_type);

    for entry in mob.loot {
        if rng.random::<f32>() < entry.chance {
            let qty = rng.random_range(entry.min_qty..=entry.max_qty);
            let mut new_item = make_item(entry.item, qty);
            // Cap rarity to mob type limit
            if new_item.rarity > max_rarity {
                new_item.rarity = max_rarity.clone();
            }
            if let Some(existing) = inventory.iter_mut().find(|i| i.name == entry.item) {
                existing.qty += qty;
            } else {
                inventory.push(new_item.clone());
            }
            gained.push(new_item);
        }
    }

    let mult = damage_multiplier(char_save.level, mob.level);
    let base_dmg = (mob.atk as i32 - effective_def as i32).max(1);
    let damage = base_dmg * mult;
    char_save.hp -= damage;

    if char_save.hp <= 0 {
        let gold_loss = (char_save.gold / 10).max(1);
        char_save.gold = char_save.gold.saturating_sub(gold_loss);
        char_save.hp = char_save.max_hp;
        return CombatOutcome {
            loot: vec![],
            xp_gained: 0,
            gold_gained: 0,
            damage_taken: damage,
            died: true,
            has_epic_drop: false,
            epic_item_name: None,
        };
    }

    char_save.xp += mob.xp_reward;
    char_save.gold += mob.gold_reward;

    let epic_item = gained.iter()
        .find(|i| i.rarity >= Rarity::Epic)
        .map(|i| i.name.clone());
    let has_epic = epic_item.is_some();

    CombatOutcome {
        loot: gained,
        xp_gained: mob.xp_reward,
        gold_gained: mob.gold_reward,
        damage_taken: damage,
        died: false,
        has_epic_drop: has_epic,
        epic_item_name: epic_item,
    }
}
