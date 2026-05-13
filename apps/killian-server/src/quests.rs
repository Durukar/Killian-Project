use killian_protocol::{Quest, QuestObjective};

use crate::persistence::SavedQuest;

pub struct QuestDef {
    pub id: &'static str,
    pub title: &'static str,
    pub giver: &'static str,
    pub zone: &'static str,
    pub reward_xp: u32,
    pub reward_gold: u32,
}

pub fn all_quest_defs() -> &'static [QuestDef] {
    &[
        QuestDef { id: "matar_goblins",    title: "Exterminio de Goblins",     giver: "Ferreiro Hruk",   zone: "cidade", reward_xp: 100, reward_gold: 50  },
        QuestDef { id: "madeira_mercador", title: "Fornecedor de Madeira",     giver: "Mercador Tomas",  zone: "cidade", reward_xp: 80,  reward_gold: 40  },
        QuestDef { id: "purgacao_mina",    title: "Purgacao da Mina",          giver: "Guarda Real",     zone: "cidade", reward_xp: 150, reward_gold: 60  },
        QuestDef { id: "senhor_trevas",    title: "O Senhor das Trevas",       giver: "Sabia Anciana",   zone: "cidade", reward_xp: 500, reward_gold: 200 },
    ]
}

fn objective_for(id: &str, done: u32) -> QuestObjective {
    match id {
        "matar_goblins"    => QuestObjective::Kill   { mob_id: "goblin".into(),          mob_name: "Goblin".into(),            required: 5, done },
        "madeira_mercador" => QuestObjective::Gather  { item_name: "Madeira".into(),                                           required: 15, done },
        "purgacao_mina"    => QuestObjective::Kill   { mob_id: "esqueleto".into(),       mob_name: "Esqueleto".into(),         required: 3, done },
        "senhor_trevas"    => QuestObjective::Kill   { mob_id: "senhor_sombras".into(),  mob_name: "Senhor das Sombras".into(), required: 1, done },
        _ => QuestObjective::Kill { mob_id: "".into(), mob_name: "?".into(), required: 1, done: 0 },
    }
}

fn quest_from_def(def: &QuestDef, done: u32) -> Quest {
    let objective = objective_for(def.id, done);
    let can_turn_in = match &objective {
        QuestObjective::Kill   { required, done, .. } => done >= required,
        QuestObjective::Gather { required, done, .. } => done >= required,
    };
    Quest {
        id: def.id.to_string(),
        title: def.title.to_string(),
        giver: def.giver.to_string(),
        objective,
        reward_xp: def.reward_xp,
        reward_gold: def.reward_gold,
        can_turn_in,
    }
}

pub fn build_quest_list(active: &[SavedQuest]) -> Vec<Quest> {
    active.iter()
        .filter_map(|sq| {
            all_quest_defs()
                .iter()
                .find(|d| d.id == sq.id)
                .map(|def| quest_from_def(def, sq.done))
        })
        .collect()
}

pub fn accept_quest(active: &mut Vec<SavedQuest>, quest_id: &str) -> bool {
    if all_quest_defs().iter().any(|d| d.id == quest_id) && !active.iter().any(|q| q.id == quest_id) {
        active.push(SavedQuest { id: quest_id.to_string(), done: 0 });
        true
    } else {
        false
    }
}

/// Returns (xp_reward, gold_reward) if quest is complete and removed, else None
pub fn turn_in_quest(active: &mut Vec<SavedQuest>, quest_id: &str) -> Option<(u32, u32)> {
    let def = all_quest_defs().iter().find(|d| d.id == quest_id)?;
    let sq  = active.iter().find(|q| q.id == quest_id)?;
    let done = sq.done;
    let objective = objective_for(quest_id, done);
    let complete = match &objective {
        QuestObjective::Kill   { required, done, .. } => done >= required,
        QuestObjective::Gather { required, done, .. } => done >= required,
    };
    if complete {
        active.retain(|q| q.id != quest_id);
        Some((def.reward_xp, def.reward_gold))
    } else {
        None
    }
}

/// Called on mob kill — increments matching kill quests
pub fn on_mob_killed(active: &mut Vec<SavedQuest>, mob_id: &str) {
    for sq in active.iter_mut() {
        let obj = objective_for(&sq.id, sq.done);
        if let QuestObjective::Kill { mob_id: req_mob, required, .. } = obj {
            if req_mob == mob_id && sq.done < required {
                sq.done += 1;
            }
        }
    }
}

/// Called on gather — increments matching gather quests
pub fn on_items_gathered(active: &mut Vec<SavedQuest>, item_name: &str, qty: u32) {
    for sq in active.iter_mut() {
        let obj = objective_for(&sq.id, sq.done);
        if let QuestObjective::Gather { item_name: req_item, required, .. } = obj {
            if req_item == item_name && sq.done < required {
                sq.done = (sq.done + qty).min(required);
            }
        }
    }
}

/// Returns which NPC has a quest the player can accept (not already active)
pub fn available_quest_for_npc(npc_name: &str, active: &[SavedQuest]) -> Option<&'static QuestDef> {
    all_quest_defs().iter().find(|d| {
        d.giver == npc_name && !active.iter().any(|q| q.id == d.id)
    })
}

/// Returns active quest from this NPC (for turn-in)
pub fn active_quest_for_npc<'a>(npc_name: &str, active: &'a [SavedQuest]) -> Option<&'a SavedQuest> {
    all_quest_defs().iter()
        .find(|d| d.giver == npc_name)
        .and_then(|def| active.iter().find(|q| q.id == def.id))
}
