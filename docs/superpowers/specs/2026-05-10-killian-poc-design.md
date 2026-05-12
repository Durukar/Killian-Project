# Killian MMORPG POC — Design Spec

**Goal:** TUI client com layout estilo LazyGit (painéis empilhados à esquerda, chat à direita) conectado via WebSocket a um servidor com personagem, inventário e sistema de craft funcional.

## Telas

### Connect Screen
- Campo Nick (sem senha)
- Campo Server (ex: `ws://127.0.0.1:7001`)
- Tab alterna foco, Enter conecta

### Game Screen (LazyGit style)
```
┌─Killian──────────────────────────────────────────────────────────┐
│ player @ ws://127.0.0.1:7001          1:Personagem 2:Inv 3:Craft │
├─[1] PERSONAGEM──────┬─CHAT────────────────────────────────────── ┤
│ Classe: Aventureiro │ [sistema] player entrou                    │
│ Nivel: 1            │ player: oi                                 │
│ HP:  100/100        │                                            │
│ MP:   35/35         │                                            │
│ Ouro: 150           │                                            │
├─[2] INVENTARIO──────┤                                            │
│ ▶ Pocao Pequena x3  │                                            │
│   Espada Curta +0   │                                            │
│   Madeira x12       │                                            │
├─[3] CRAFT───────────┤                                            │
│ ▶ Pocao Media       │                                            │
│   Espada Longa      │                                            │
│   Escudo de Madeira │                                            │
└─────────────────────┴────────────────────────────────────────────┘
 > _   ↑↓: navegar | Enter: craftar | 1-3: painel | Tab: chat | q: sair
```

## Protocolo WebSocket

### ClientMsg (novo)
- `Craft { recipe_id: String }` — solicita craft de uma receita

### ServerMsg (novos)
- `CharacterUpdate { class_name, level, hp, max_hp, mp, max_mp, gold }` — enviado ao conectar
- `InventoryUpdate { items: Vec<InventoryItem> }` — enviado ao conectar e após craft
- `RecipesUpdate { recipes: Vec<Recipe> }` — enviado ao conectar
- `CraftResult { success: bool, message: String }` — resposta ao craft

### Structs
```
InventoryItem { name: String, qty: u32 }
Recipe { id: String, name: String, ingredients: Vec<InventoryItem>, result: InventoryItem }
```

## Craft (hardcoded no servidor)

| Receita | Ingredientes | Resultado |
|---------|-------------|-----------|
| Pocao Media | Pocao Pequena x2 | Pocao Media x1 |
| Espada Longa | Madeira x5 + Pedra x3 | Espada Longa x1 |
| Escudo de Madeira | Madeira x8 | Escudo de Madeira x1 |

## Navegação

| Tecla | Ação |
|-------|------|
| `1` | Foca painel Personagem |
| `2` | Foca painel Inventário |
| `3` | Foca painel Craft |
| `Tab` | Cicla painéis |
| `↑` / `↓` | Move cursor em Inventário ou Craft |
| `Enter` | Crafta item selecionado (quando em Craft) |
| Qualquer char | Digita no chat input |
| `Esc` / `Ctrl+C` | Sai |
