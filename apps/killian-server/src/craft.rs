use killian_protocol::{InventoryItem, Rarity, Recipe};

use crate::items::make_item;
use crate::persistence::CharacterSave;

fn recipe(id: &str, name: &str, prof: &str, min_lv: u32, ingredients: Vec<InventoryItem>, result: InventoryItem) -> Recipe {
    Recipe {
        id: id.to_string(),
        name: name.to_string(),
        profession: prof.to_string(),
        min_prof_level: min_lv,
        ingredients,
        result,
    }
}

fn ing(name: &str, qty: u32) -> InventoryItem {
    InventoryItem { name: name.to_string(), qty, ..Default::default() }
}

pub fn all_recipes() -> Vec<Recipe> {
    vec![
        // ── ALQUIMISTA ────────────────────────────────────────
        recipe("pocao_media",        "Pocao Media",        "alquimista", 1, vec![ing("Pocao Pequena", 2)],                              make_item("Pocao Media", 1)),
        recipe("pocao_grande",       "Pocao Grande",       "alquimista", 3, vec![ing("Pocao Media", 2), ing("Erva Rara", 1)],           make_item("Pocao Grande", 1)),
        recipe("elixir_batalha",     "Elixir de Batalha",  "alquimista", 6, vec![ing("Pocao Grande", 2), ing("Cristal de Mana", 1)],    make_item("Elixir de Batalha", 1)),

        // ── FERREIRO ─────────────────────────────────────────
        recipe("espada_longa",       "Espada Longa",       "ferreiro",   1, vec![ing("Pedra", 3), ing("Madeira", 2)],                   make_item("Espada Longa", 1)),
        recipe("escudo_madeira",     "Escudo de Madeira",  "ferreiro",   1, vec![ing("Madeira", 8)],                                    make_item("Escudo de Madeira", 1)),
        recipe("espada_ferro",       "Espada de Ferro",    "ferreiro",   4, vec![ing("Mineral de Ferro", 4), ing("Madeira", 2)],        make_item("Espada de Ferro", 1)),
        recipe("coraca_ferro",       "Coraca de Ferro",    "ferreiro",   4, vec![ing("Mineral de Ferro", 6), ing("Pedra", 4)],          make_item("Coraca de Ferro", 1)),
        recipe("armadura_epica",     "Armadura Epica",     "ferreiro",   8, vec![ing("Mineral de Ferro", 12), ing("Pedra das Sombras", 4), ing("Cristal de Mana", 2)], make_item("Armadura Epica", 1)),

        // ── LENHADOR (ferramentas) ────────────────────────────
        recipe("machado_simples",    "Machado Simples",    "lenhador",   1, vec![ing("Madeira", 4), ing("Pedra", 2)],                   make_item("Machado Simples", 1)),
        recipe("machado_ferro",      "Machado de Ferro",   "lenhador",   4, vec![ing("Mineral de Ferro", 3), ing("Madeira", 3)],        make_item("Machado de Ferro", 1)),

        // ── MINERADOR (ferramentas) ───────────────────────────
        recipe("picareta_simples",   "Picareta Simples",   "minerador",  1, vec![ing("Madeira", 3), ing("Pedra", 4)],                   make_item("Picareta Simples", 1)),
        recipe("picareta_ferro",     "Picareta de Ferro",  "minerador",  4, vec![ing("Mineral de Ferro", 4), ing("Madeira", 2)],        make_item("Picareta de Ferro", 1)),
    ]
}

/// Recipes visible to a player based on their profession and level
pub fn recipes_for_player(char_save: &CharacterSave) -> Vec<Recipe> {
    let all = all_recipes();
    all.into_iter()
        .filter(|r| {
            r.profession == char_save.profession
                && r.min_prof_level <= char_save.profession_level
        })
        .collect()
}

pub fn can_craft(inventory: &[InventoryItem], recipe: &Recipe) -> bool {
    recipe.ingredients.iter().all(|ing| {
        inventory.iter().any(|item| item.name == ing.name && item.qty >= ing.qty)
    })
}

/// Returns the quality rarity of a crafted item based on profession level and craft_power tool bonus
pub fn craft_quality(prof_level: u32, craft_power: u32) -> Rarity {
    let effective = prof_level + craft_power * 2;
    match effective {
        0..=2  => Rarity::Common,
        3..=4  => Rarity::Uncommon,
        5..=6  => Rarity::Rare,
        7..=8  => Rarity::Epic,
        _      => Rarity::RefinedEpic,
    }
}

pub fn apply_craft(inventory: &mut Vec<InventoryItem>, recipe: &Recipe, quality: Rarity) -> InventoryItem {
    for ing in &recipe.ingredients {
        if let Some(item) = inventory.iter_mut().find(|i| i.name == ing.name) {
            item.qty -= ing.qty;
        }
    }
    inventory.retain(|i| i.qty > 0);

    let mut result = recipe.result.clone();
    result.rarity = quality;

    if let Some(existing) = inventory.iter_mut().find(|i| i.name == result.name) {
        existing.qty += result.qty;
    } else {
        inventory.push(result.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_craft_pocao_media_when_has_ingredients() {
        let inv = vec![InventoryItem { name: "Pocao Pequena".to_string(), qty: 3, ..Default::default() }];
        let recipe = all_recipes().into_iter().find(|r| r.id == "pocao_media").unwrap();
        assert!(can_craft(&inv, &recipe));
    }

    #[test]
    fn craft_quality_scales_with_level() {
        assert_eq!(craft_quality(1, 0), Rarity::Common);
        assert_eq!(craft_quality(5, 0), Rarity::Rare);
        assert_eq!(craft_quality(8, 0), Rarity::Epic);
        assert_eq!(craft_quality(10, 0), Rarity::RefinedEpic);
    }
}
