use killian_protocol::{InventoryItem, ItemType, Rarity};

pub fn make_item(name: &str, qty: u32) -> InventoryItem {
    match name {
        // ── Materiais de coleta básica ─────────────────────────
        "Madeira" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Pedra"   => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Galho"   => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },

        // Materiais avançados (zonas de alta dificuldade)
        "Mineral de Ferro"   => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Cristal de Mana"    => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Pedra das Sombras"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Erva Rara"          => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },
        "Madeira Negra"      => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Material, ..Default::default() },

        // ── Ferramentas ────────────────────────────────────────
        "Machado Simples"   => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, gather_power: 1, ..Default::default() },
        "Machado de Ferro"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, gather_power: 2, ..Default::default() },
        "Picareta Simples"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, gather_power: 1, ..Default::default() },
        "Picareta de Ferro" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, gather_power: 2, ..Default::default() },
        "Caldeirão Simples" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, craft_power: 1, ..Default::default() },
        "Caldeirão Arcano"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, craft_power: 2, ..Default::default() },
        "Martelo do Ferreiro" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Tool, craft_power: 1, ..Default::default() },

        // ── Consumíveis ────────────────────────────────────────
        "Pocao Pequena"     => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Consumable, hp_restore: 30,  ..Default::default() },
        "Pocao Media"       => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Consumable, hp_restore: 60,  ..Default::default() },
        "Pocao Grande"      => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Consumable, hp_restore: 120, rarity: Rarity::Uncommon, ..Default::default() },
        "Elixir de Batalha" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Consumable, hp_restore: 200, rarity: Rarity::Rare, ..Default::default() },

        // ── Armas ──────────────────────────────────────────────
        "Espada Curta"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Weapon, str_bonus: 3, ..Default::default() },
        "Espada Longa"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Weapon, str_bonus: 5, rarity: Rarity::Uncommon, ..Default::default() },
        "Espada de Ferro" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Weapon, str_bonus: 8, rarity: Rarity::Rare, ..Default::default() },
        "Lanca das Sombras" => InventoryItem {
            name: name.to_string(), qty, item_type: ItemType::Weapon,
            rarity: Rarity::RefinedEpic,
            str_bonus: 15, agi_bonus: 8,
            ..Default::default()
        },

        // ── Armaduras ─────────────────────────────────────────
        "Escudo de Madeira" => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Armor, def_bonus: 3, ..Default::default() },
        "Coraca de Ferro"   => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Armor, def_bonus: 7, rarity: Rarity::Rare, ..Default::default() },
        "Botas de Combate"  => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Armor, agi_bonus: 4, ..Default::default() },
        "Luvas de Couro"    => InventoryItem { name: name.to_string(), qty, item_type: ItemType::Armor, agi_bonus: 2, ..Default::default() },
        "Manto das Sombras" => InventoryItem {
            name: name.to_string(), qty, item_type: ItemType::Armor,
            rarity: Rarity::Rare,
            def_bonus: 5, agi_bonus: 6,
            ..Default::default()
        },
        "Armadura Epica"    => InventoryItem {
            name: name.to_string(), qty, item_type: ItemType::Armor,
            rarity: Rarity::Epic,
            def_bonus: 14, vit_bonus: 5,
            ..Default::default()
        },

        // ── Anéis / Acessórios ─────────────────────────────────
        "Amuleto Antigo"    => InventoryItem {
            name: name.to_string(), qty, item_type: ItemType::Ring,
            rarity: Rarity::Epic,
            str_bonus: 3, def_bonus: 3, agi_bonus: 3, vit_bonus: 3,
            ..Default::default()
        },
        "Anel do Pantano"   => InventoryItem {
            name: name.to_string(), qty, item_type: ItemType::Ring,
            rarity: Rarity::Rare,
            vit_bonus: 4, def_bonus: 3,
            ..Default::default()
        },

        _ => InventoryItem { name: name.to_string(), qty, ..Default::default() },
    }
}

pub fn item_hp_restore(name: &str) -> u32 {
    match name {
        "Pocao Pequena"     => 30,
        "Pocao Media"       => 60,
        "Pocao Grande"      => 120,
        "Elixir de Batalha" => 200,
        _ => 0,
    }
}

pub fn item_gather_power(inventory: &[killian_protocol::InventoryItem], equipped: &[String]) -> u32 {
    equipped.iter()
        .filter_map(|name| inventory.iter().find(|i| &i.name == name))
        .map(|i| i.gather_power)
        .max()
        .unwrap_or(0)
}

pub fn item_craft_power(inventory: &[killian_protocol::InventoryItem], equipped: &[String]) -> u32 {
    equipped.iter()
        .filter_map(|name| inventory.iter().find(|i| &i.name == name))
        .map(|i| i.craft_power)
        .max()
        .unwrap_or(0)
}
