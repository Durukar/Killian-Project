use killian_protocol::{InventoryItem, ItemType};

pub fn make_item(name: &str, qty: u32) -> InventoryItem {
    match name {
        "Madeira" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Material,
            ..Default::default()
        },
        "Pedra" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Material,
            ..Default::default()
        },
        "Pocao Pequena" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Consumable,
            hp_restore: 30,
            ..Default::default()
        },
        "Pocao Media" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Consumable,
            hp_restore: 60,
            ..Default::default()
        },
        "Espada Curta" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Weapon,
            str_bonus: 3,
            ..Default::default()
        },
        "Espada Longa" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Weapon,
            str_bonus: 5,
            ..Default::default()
        },
        "Escudo de Madeira" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Armor,
            def_bonus: 3,
            ..Default::default()
        },
        "Coraca de Ferro" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Armor,
            def_bonus: 7,
            ..Default::default()
        },
        "Botas de Combate" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Armor,
            agi_bonus: 4,
            ..Default::default()
        },
        "Amuleto Antigo" => InventoryItem {
            name: name.to_string(), qty,
            item_type: ItemType::Ring,
            rarity: killian_protocol::Rarity::Rare,
            str_bonus: 2,
            def_bonus: 2,
            agi_bonus: 2,
            vit_bonus: 2,
            ..Default::default()
        },
        _ => InventoryItem { name: name.to_string(), qty, ..Default::default() },
    }
}

pub fn item_hp_restore(name: &str) -> u32 {
    match name {
        "Pocao Pequena" => 30,
        "Pocao Media"   => 60,
        _ => 0,
    }
}
