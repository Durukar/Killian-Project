use killian_protocol::InventoryItem;
use rand::Rng;

pub struct GatherAction {
    pub id: &'static str,
    pub name: &'static str,
    pub location: &'static str,
    pub yield_item: &'static str,
    pub min_qty: u32,
    pub max_qty: u32,
}

pub fn all_gather_actions() -> &'static [GatherAction] {
    &[
        GatherAction { id: "coletar_madeira",  name: "Coletar Madeira",  location: "Floresta", yield_item: "Madeira",       min_qty: 2, max_qty: 4 },
        GatherAction { id: "coletar_galhos",   name: "Coletar Galhos",   location: "Floresta", yield_item: "Madeira",       min_qty: 1, max_qty: 2 },
        GatherAction { id: "minerar_pedra",    name: "Minerar Pedra",    location: "Mina",     yield_item: "Pedra",         min_qty: 2, max_qty: 3 },
        GatherAction { id: "minerar_mineral",  name: "Minerar Mineral",  location: "Mina",     yield_item: "Pedra",         min_qty: 4, max_qty: 6 },
        GatherAction { id: "colher_ervas",     name: "Colher Ervas",     location: "Campos",   yield_item: "Pocao Pequena", min_qty: 1, max_qty: 1 },
    ]
}

pub fn apply_gather(inventory: &mut Vec<InventoryItem>, action: &GatherAction) -> Vec<InventoryItem> {
    let qty = rand::rng().random_range(action.min_qty..=action.max_qty);
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == action.yield_item) {
        existing.qty += qty;
    } else {
        inventory.push(InventoryItem { name: action.yield_item.to_string(), qty });
    }
    vec![InventoryItem { name: action.yield_item.to_string(), qty }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_gather_adds_item_to_empty_inventory() {
        let mut inv = vec![];
        let action = &all_gather_actions()[0]; // coletar_madeira
        let yielded = apply_gather(&mut inv, action);
        assert!(!inv.is_empty());
        assert_eq!(inv[0].name, "Madeira");
        assert!(inv[0].qty >= action.min_qty && inv[0].qty <= action.max_qty);
        assert_eq!(yielded[0].name, "Madeira");
    }

    #[test]
    fn apply_gather_stacks_on_existing_item() {
        let mut inv = vec![InventoryItem { name: "Madeira".to_string(), qty: 5 }];
        let action = &all_gather_actions()[0]; // coletar_madeira
        apply_gather(&mut inv, action);
        assert_eq!(inv.len(), 1);
        assert!(inv[0].qty >= 5 + action.min_qty);
    }

    #[test]
    fn apply_gather_creates_new_item_when_not_present() {
        let mut inv = vec![InventoryItem { name: "Pedra".to_string(), qty: 3 }];
        let action = &all_gather_actions()[0]; // coletar_madeira (Madeira)
        apply_gather(&mut inv, action);
        assert_eq!(inv.len(), 2);
        assert!(inv.iter().any(|i| i.name == "Madeira"));
    }
}
