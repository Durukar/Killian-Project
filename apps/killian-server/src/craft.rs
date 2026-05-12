use killian_protocol::{InventoryItem, Recipe};

use crate::items::make_item;

pub fn all_recipes() -> Vec<Recipe> {
    vec![
        Recipe {
            id: "pocao_media".to_string(),
            name: "Pocao Media".to_string(),
            ingredients: vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 2, ..Default::default() }],
            result: make_item("Pocao Media", 1),
        },
        Recipe {
            id: "espada_longa".to_string(),
            name: "Espada Longa".to_string(),
            ingredients: vec![
                InventoryItem { name: "Madeira".to_string(), qty: 5, ..Default::default() },
                InventoryItem { name: "Pedra".to_string(), qty: 3, ..Default::default() },
            ],
            result: make_item("Espada Longa", 1),
        },
        Recipe {
            id: "escudo_madeira".to_string(),
            name: "Escudo de Madeira".to_string(),
            ingredients: vec![InventoryItem { name: "Madeira".to_string(), qty: 8, ..Default::default() }],
            result: make_item("Escudo de Madeira", 1),
        },
    ]
}

pub fn can_craft(inventory: &[InventoryItem], recipe: &Recipe) -> bool {
    recipe.ingredients.iter().all(|ing| {
        inventory
            .iter()
            .any(|item| item.name == ing.name && item.qty >= ing.qty)
    })
}

pub fn apply_craft(inventory: &mut Vec<InventoryItem>, recipe: &Recipe) {
    for ing in &recipe.ingredients {
        if let Some(item) = inventory.iter_mut().find(|i| i.name == ing.name) {
            item.qty -= ing.qty;
        }
    }
    inventory.retain(|i| i.qty > 0);
    if let Some(existing) = inventory.iter_mut().find(|i| i.name == recipe.result.name) {
        existing.qty += recipe.result.qty;
    } else {
        inventory.push(recipe.result.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_inventory() -> Vec<InventoryItem> {
        vec![
            InventoryItem { name: "Pocao Pequena".to_string(), qty: 3, ..Default::default() },
            InventoryItem { name: "Madeira".to_string(), qty: 12, ..Default::default() },
            InventoryItem { name: "Pedra".to_string(), qty: 6, ..Default::default() },
        ]
    }

    #[test]
    fn can_craft_pocao_media_when_has_ingredients() {
        let inv = base_inventory();
        let recipe = &all_recipes()[0];
        assert!(can_craft(&inv, recipe));
    }

    #[test]
    fn cannot_craft_when_missing_ingredient() {
        let inv = vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 1, ..Default::default() }];
        let recipe = &all_recipes()[0];
        assert!(!can_craft(&inv, recipe));
    }

    #[test]
    fn apply_craft_consumes_ingredients_and_adds_result() {
        let mut inv = base_inventory();
        let recipe = &all_recipes()[0];
        apply_craft(&mut inv, recipe);
        let pocao_p = inv.iter().find(|i| i.name == "Pocao Pequena").unwrap();
        assert_eq!(pocao_p.qty, 1);
        let pocao_m = inv.iter().find(|i| i.name == "Pocao Media").unwrap();
        assert_eq!(pocao_m.qty, 1);
    }

    #[test]
    fn apply_craft_removes_item_when_qty_reaches_zero() {
        let mut inv = vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 2, ..Default::default() }];
        let recipe = &all_recipes()[0];
        apply_craft(&mut inv, recipe);
        assert!(!inv.iter().any(|i| i.name == "Pocao Pequena"));
    }
}
