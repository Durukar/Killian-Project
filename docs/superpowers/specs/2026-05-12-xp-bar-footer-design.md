# XP Bar Footer — Design Spec

**Goal:** Exibir uma barra de progresso de XP logo abaixo do header, como uma linha fixa de 1 linha de altura.

**Escopo:** Apenas `apps/killian-client/src/view.rs`. Sem mudanças de protocolo, model ou view_model.

---

## Layout

O layout atual do `Screen::Game` usa dois constraints:

```
[ Header    — Length(3) ]
[ Conteúdo  — Min(0)    ]
```

Passa a ser:

```
[ Header    — Length(3) ]
[ XP Bar    — Length(1) ]
[ Conteúdo  — Min(0)    ]
```

A XP bar é renderizada como um `Paragraph` sem borda, ocupando toda a largura do terminal.

---

## Conteúdo visual

```
 Lv.3  ████████████░░░░░░░░░░░░  340/500 XP
```

- **Label esquerdo:** `Lv.X` — Yellow Bold
- **Barra central:** blocos `█` (preenchido, Cyan) e `░` (vazio, DarkGray); largura = largura total menos ~22 colunas reservadas para os labels
- **Label direito:** `CUR/NEXT XP` — DarkGray
- Sem borda, sem padding extra — linha única rente ao header

---

## Cálculo da barra

```
ratio     = character.xp as f64 / character.xp_next as f64  (clamped 0.0..=1.0)
bar_width = area.width.saturating_sub(22) as usize
filled    = (ratio * bar_width as f64) as usize
empty     = bar_width - filled
bar_str   = "█".repeat(filled) + &"░".repeat(empty)
```

---

## Casos de borda

| Situação | Comportamento |
|---|---|
| `character` é `None` | Linha em branco (ainda conectando) |
| `xp_next == 0` | Barra cheia (edge case de dados inválidos) |
| Terminal muito estreito (< 22 cols) | Barra omitida, só labels |

---

## Arquivos modificados

| Arquivo | Mudança |
|---|---|
| `apps/killian-client/src/view.rs` | Novo constraint `Length(1)` no layout de Game; nova função `render_xp_bar()` |

Nenhum outro arquivo é tocado.
