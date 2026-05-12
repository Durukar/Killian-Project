use killian_protocol::InventoryItem;
use rand::Rng;

use crate::items::make_item;
use crate::persistence::CharacterSave;

pub struct LootEntry {
    pub item: &'static str,
    pub min_qty: u32,
    pub max_qty: u32,
    pub chance: f32,
}

#[allow(dead_code)]
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
    pub loot: &'static [LootEntry],
}

pub fn all_mobs() -> &'static [Mob] {
    &[
        Mob {
            id: "goblin", name: "Goblin", zone: "floresta",
            level: 1, hp: 30, atk: 5, def: 2, xp_reward: 20, gold_reward: 5,
            loot: &[
                LootEntry { item: "Madeira",       min_qty: 1, max_qty: 3, chance: 0.8 },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.3 },
            ],
        },
        Mob {
            id: "lobo", name: "Lobo", zone: "floresta",
            level: 2, hp: 45, atk: 8, def: 3, xp_reward: 35, gold_reward: 8,
            loot: &[
                LootEntry { item: "Madeira",       min_qty: 2, max_qty: 4, chance: 0.6 },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5 },
            ],
        },
        Mob {
            id: "morcego", name: "Morcego", zone: "mina",
            level: 2, hp: 25, atk: 6, def: 1, xp_reward: 25, gold_reward: 4,
            loot: &[
                LootEntry { item: "Pedra", min_qty: 1, max_qty: 2, chance: 0.9 },
            ],
        },
        Mob {
            id: "esqueleto", name: "Esqueleto", zone: "mina",
            level: 3, hp: 60, atk: 10, def: 5, xp_reward: 50, gold_reward: 12,
            loot: &[
                LootEntry { item: "Pedra",         min_qty: 2, max_qty: 4, chance: 0.7 },
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.4 },
            ],
        },
        Mob {
            id: "sanguessuga", name: "Sanguessuga", zone: "pantano",
            level: 2, hp: 40, atk: 9, def: 2, xp_reward: 30, gold_reward: 6,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5 },
            ],
        },
        Mob {
            id: "basilisco", name: "Basilisco", zone: "pantano",
            level: 4, hp: 90, atk: 16, def: 7, xp_reward: 80, gold_reward: 18,
            loot: &[
                LootEntry { item: "Pocao Media",   min_qty: 1, max_qty: 1, chance: 0.4 },
                LootEntry { item: "Pedra",         min_qty: 2, max_qty: 4, chance: 0.6 },
            ],
        },
        Mob {
            id: "aranha", name: "Aranha", zone: "caverna",
            level: 2, hp: 35, atk: 8, def: 2, xp_reward: 28, gold_reward: 5,
            loot: &[
                LootEntry { item: "Pedra", min_qty: 1, max_qty: 2, chance: 0.8 },
            ],
        },
        Mob {
            id: "golem", name: "Golem de Pedra", zone: "caverna",
            level: 5, hp: 120, atk: 18, def: 12, xp_reward: 110, gold_reward: 25,
            loot: &[
                LootEntry { item: "Pedra",       min_qty: 3, max_qty: 6, chance: 0.9 },
                LootEntry { item: "Pocao Media", min_qty: 1, max_qty: 1, chance: 0.3 },
            ],
        },
        Mob {
            id: "javali", name: "Javali", zone: "campos",
            level: 3, hp: 70, atk: 12, def: 4, xp_reward: 55, gold_reward: 10,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 2, chance: 0.6 },
            ],
        },
        Mob {
            id: "bandido", name: "Bandido", zone: "campos",
            level: 4, hp: 80, atk: 14, def: 6, xp_reward: 70, gold_reward: 20,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5 },
                LootEntry { item: "Madeira",       min_qty: 1, max_qty: 2, chance: 0.4 },
                LootEntry { item: "Pedra",         min_qty: 1, max_qty: 2, chance: 0.4 },
            ],
        },
        Mob {
            id: "urso", name: "Urso", zone: "montanha",
            level: 4, hp: 100, atk: 17, def: 8, xp_reward: 90, gold_reward: 15,
            loot: &[
                LootEntry { item: "Pocao Media",   min_qty: 1, max_qty: 1, chance: 0.5 },
                LootEntry { item: "Madeira",       min_qty: 1, max_qty: 3, chance: 0.5 },
            ],
        },
        Mob {
            id: "gigante", name: "Gigante", zone: "montanha",
            level: 6, hp: 160, atk: 22, def: 14, xp_reward: 150, gold_reward: 35,
            loot: &[
                LootEntry { item: "Pocao Media",   min_qty: 1, max_qty: 2, chance: 0.6 },
                LootEntry { item: "Pedra",         min_qty: 3, max_qty: 6, chance: 0.7 },
            ],
        },
        Mob {
            id: "escorpiao", name: "Escorpião", zone: "deserto",
            level: 3, hp: 55, atk: 11, def: 5, xp_reward: 48, gold_reward: 12,
            loot: &[
                LootEntry { item: "Pocao Pequena", min_qty: 1, max_qty: 1, chance: 0.5 },
            ],
        },
        Mob {
            id: "drake", name: "Drake do Deserto", zone: "deserto",
            level: 6, hp: 150, atk: 24, def: 10, xp_reward: 160, gold_reward: 40,
            loot: &[
                LootEntry { item: "Pocao Media",   min_qty: 1, max_qty: 2, chance: 0.7 },
                LootEntry { item: "Madeira",       min_qty: 2, max_qty: 4, chance: 0.4 },
            ],
        },
    ]
}

pub struct CombatOutcome {
    pub loot: Vec<InventoryItem>,
    pub xp_gained: u32,
    pub gold_gained: u32,
    pub damage_taken: i32,
    pub died: bool,
}

pub fn apply_combat(
    inventory: &mut Vec<InventoryItem>,
    mob: &Mob,
    char_save: &mut CharacterSave,
) -> CombatOutcome {
    let mut rng = rand::rng();
    let mut gained = Vec::new();

    for entry in mob.loot {
        if rng.random::<f32>() < entry.chance {
            let qty = rng.random_range(entry.min_qty..=entry.max_qty);
            let new_item = make_item(entry.item, qty);
            if let Some(existing) = inventory.iter_mut().find(|i| i.name == entry.item) {
                existing.qty += qty;
            } else {
                inventory.push(new_item.clone());
            }
            gained.push(new_item);
        }
    }

    // Damage formula: max(1, mob_atk - player_def)
    let damage = (mob.atk as i32 - char_save.def_stat as i32).max(1);
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
        };
    }

    char_save.xp += mob.xp_reward;
    char_save.gold += mob.gold_reward;

    CombatOutcome {
        loot: gained,
        xp_gained: mob.xp_reward,
        gold_gained: mob.gold_reward,
        damage_taken: damage,
        died: false,
    }
}
