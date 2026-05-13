use std::path::Path;

use killian_protocol::{InventoryItem, MarketListing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEntry {
    pub id: u64,
    pub seller: String,
    pub item: InventoryItem,
    pub price: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Market {
    pub listings: Vec<MarketEntry>,
    next_id: u64,
}

const MARKET_PATH: &str = "data/market.json";

impl Market {
    pub fn load() -> Self {
        let path = Path::new(MARKET_PATH);
        if path.exists() {
            if let Ok(bytes) = std::fs::read_to_string(path) {
                if let Ok(m) = serde_json::from_str(&bytes) {
                    return m;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(MARKET_PATH, json);
        }
    }

    pub fn to_protocol(&self) -> Vec<MarketListing> {
        self.listings.iter().map(|e| MarketListing {
            id: e.id,
            seller: e.seller.clone(),
            item: e.item.clone(),
            price: e.price,
        }).collect()
    }

    /// List an item. Returns the new listing id, or an error reason.
    pub fn list_item(
        &mut self,
        seller: &str,
        item: InventoryItem,
        price: u32,
    ) -> Result<u64, &'static str> {
        if price == 0 {
            return Err("Preço deve ser maior que zero.");
        }
        if item.qty == 0 {
            return Err("Quantidade deve ser maior que zero.");
        }
        let id = self.next_id;
        self.next_id += 1;
        self.listings.push(MarketEntry {
            id,
            seller: seller.to_string(),
            item,
            price,
        });
        self.save();
        Ok(id)
    }

    /// Buy a listing. Returns (entry, gold_paid) or error reason.
    pub fn buy_item(
        &mut self,
        buyer: &str,
        listing_id: u64,
        buyer_gold: u32,
    ) -> Result<MarketEntry, &'static str> {
        let pos = self.listings.iter().position(|e| e.id == listing_id)
            .ok_or("Listagem não encontrada.")?;
        if self.listings[pos].seller == buyer {
            return Err("Você não pode comprar seu próprio item.");
        }
        let price = self.listings[pos].price;
        if buyer_gold < price {
            return Err("Ouro insuficiente.");
        }
        let entry = self.listings.remove(pos);
        self.save();
        Ok(entry)
    }

    /// Cancel own listing. Returns the item back or error.
    pub fn cancel_listing(
        &mut self,
        seller: &str,
        listing_id: u64,
    ) -> Result<MarketEntry, &'static str> {
        let pos = self.listings.iter().position(|e| e.id == listing_id)
            .ok_or("Listagem não encontrada.")?;
        if self.listings[pos].seller != seller {
            return Err("Você não pode cancelar a listagem de outro jogador.");
        }
        let entry = self.listings.remove(pos);
        self.save();
        Ok(entry)
    }
}
