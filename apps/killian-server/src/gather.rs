use killian_protocol::InventoryItem;
use rand::Rng;

use crate::items::make_item;

pub struct GatherAction {
    pub id: &'static str,
    pub name: &'static str,
    pub location: &'static str,
    pub yield_item: &'static str,
    pub min_qty: u32,
    pub max_qty: u32,
    pub min_gather_power: u32, // 0 = qualquer ferramenta
    pub profession: &'static str,
}

pub fn all_gather_actions() -> &'static [GatherAction] {
    &[
        // FLORESTA — Lenhador
        GatherAction { id: "coletar_galhos",     name: "Coletar Galhos",     location: "floresta", yield_item: "Galho",           min_qty: 2, max_qty: 4,  min_gather_power: 0, profession: "lenhador" },
        GatherAction { id: "coletar_madeira",    name: "Coletar Madeira",    location: "floresta", yield_item: "Madeira",          min_qty: 2, max_qty: 4,  min_gather_power: 0, profession: "lenhador" },
        GatherAction { id: "cortar_madeira",     name: "Cortar Madeira",     location: "floresta", yield_item: "Madeira",          min_qty: 4, max_qty: 8,  min_gather_power: 1, profession: "lenhador" },
        GatherAction { id: "coletar_madeira_negra", name: "Madeira Negra",  location: "floresta", yield_item: "Madeira Negra",    min_qty: 1, max_qty: 2,  min_gather_power: 2, profession: "lenhador" },

        // MINA — Minerador
        GatherAction { id: "minerar_pedra",      name: "Minerar Pedra",      location: "mina",     yield_item: "Pedra",            min_qty: 2, max_qty: 3,  min_gather_power: 0, profession: "minerador" },
        GatherAction { id: "minerar_mineral",    name: "Minerar Mineral",    location: "mina",     yield_item: "Pedra",            min_qty: 4, max_qty: 6,  min_gather_power: 1, profession: "minerador" },
        GatherAction { id: "extrair_ferro",      name: "Extrair Ferro",      location: "mina",     yield_item: "Mineral de Ferro", min_qty: 1, max_qty: 3,  min_gather_power: 1, profession: "minerador" },
        GatherAction { id: "cristal_mana",       name: "Cristal de Mana",    location: "mina",     yield_item: "Cristal de Mana",  min_qty: 1, max_qty: 2,  min_gather_power: 2, profession: "minerador" },

        // PANTANO — Alquimista
        GatherAction { id: "colher_ervas",       name: "Colher Ervas",       location: "pantano",  yield_item: "Pocao Pequena",    min_qty: 1, max_qty: 1,  min_gather_power: 0, profession: "alquimista" },
        GatherAction { id: "colher_erva_rara",   name: "Colher Erva Rara",   location: "pantano",  yield_item: "Erva Rara",        min_qty: 1, max_qty: 1,  min_gather_power: 1, profession: "alquimista" },

        // CAMPOS — Alquimista
        GatherAction { id: "colher_plantas",     name: "Colher Plantas",     location: "campos",   yield_item: "Pocao Pequena",    min_qty: 1, max_qty: 2,  min_gather_power: 0, profession: "alquimista" },

        // TOCA DAS SOMBRAS — Minerador avançado
        GatherAction { id: "pedra_sombras",      name: "Pedra das Sombras",  location: "toca_das_sombras", yield_item: "Pedra das Sombras", min_qty: 1, max_qty: 2, min_gather_power: 2, profession: "minerador" },
    ]
}

pub struct GatherResult {
    pub items: Vec<InventoryItem>,
    pub blocked: Option<&'static str>, // None = ok, Some(reason) = bloqueado
}

pub fn apply_gather(
    inventory: &mut Vec<InventoryItem>,
    action: &GatherAction,
    gather_power: u32,
) -> GatherResult {
    if gather_power < action.min_gather_power {
        return GatherResult {
            items: vec![],
            blocked: Some("Ferramenta inadequada para esta coleta."),
        };
    }

    let bonus_qty = gather_power.saturating_sub(action.min_gather_power);
    let max_qty = action.max_qty + bonus_qty;

    let qty = rand::rng().random_range(action.min_qty..=max_qty);
    let new_item = make_item(action.yield_item, qty);
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == action.yield_item) {
        existing.qty += qty;
    } else {
        inventory.push(new_item.clone());
    }
    GatherResult { items: vec![new_item], blocked: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_gather_adds_item_to_empty_inventory() {
        let mut inv = vec![];
        let action = &all_gather_actions()[0];
        let result = apply_gather(&mut inv, action, 0);
        assert!(result.blocked.is_none());
        assert!(!inv.is_empty());
    }

    #[test]
    fn apply_gather_blocked_without_tool() {
        let mut inv = vec![];
        let action = all_gather_actions().iter().find(|a| a.min_gather_power > 0).unwrap();
        let result = apply_gather(&mut inv, action, 0);
        assert!(result.blocked.is_some());
    }

    #[test]
    fn apply_gather_stacks_on_existing_item() {
        let mut inv = vec![InventoryItem { name: "Madeira".to_string(), qty: 5, ..Default::default() }];
        let action = all_gather_actions().iter().find(|a| a.id == "coletar_madeira").unwrap();
        apply_gather(&mut inv, action, 0);
        assert_eq!(inv.len(), 1);
        assert!(inv[0].qty >= 5 + action.min_qty);
    }
}
