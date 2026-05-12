# Killian Online — Game Design Document

> **Versão:** 0.1 — Core Loop  
> **Status:** Em desenvolvimento ativo  
> **Plataforma:** Terminal (TUI) — Rust + ratatui  
> **Última atualização:** 2026-05-12

---

## Visão

**Tagline:** *"Você é o que você faz."*

Killian Online é um MMORPG de terminal onde a identidade do jogador é definida pelas suas ações, não por uma classe escolhida na criação do personagem. Um jogador que passa o tempo minerando e forjando é um ferreiro. Um que combate dungeons é um guerreiro. A árvore de talentos reflete o que você faz.

A economia não é um sistema auxiliar — é o coração do mundo. Guerreiros precisam de ferreiros. Ferreiros precisam de coletores. Coletores precisam de proteção. Essa interdependência cria comunidade orgânica.

**Referências:** Albion Online (progressão por atividade, economia player-driven), World of Warcraft (progressão épica, guilds, dungeons, raids), The Crims (economy browser-based).

---

## Pilares de Design

| Pilar | Descrição |
|-------|-----------|
| **Atividade define identidade** | Não há classe fixa. O que você faz molda quem você é. |
| **Economia é o jogo** | NPCs não vendem nada relevante. Tudo vem de jogadores. |
| **Poder coletivo** | O conteúdo mais rico exige organização. Guilds importam. |
| **Progressão visível** | Cada ação te deixa mensuradamente mais forte. |

---

## Mundo — Aldenmoor

**Aldenmoor** é um continente de fantasia medieval dividido em zonas com recursos, perigos e histórias próprias. O lore é expandido conforme o jogo cresce — a ambientação inicial é propositalmente clássica para que os jogadores projetem suas próprias histórias.

### Zonas planejadas

| Zona | Tipo | Recursos | Perigo |
|------|------|----------|--------|
| Campos de Aldenmoor | Segura | Ervas, Couro | Baixo |
| Floresta de Everfen | Segura | Madeira, Galhos | Baixo |
| Minas de Greyrock | Intermediária | Pedra, Minério | Médio |
| Pântano de Mirefall | Intermediária | Ervas raras, Veneno | Médio |
| Ruínas de Ashenveil | Perigosa | Materiais raros | Alto |
| Cidadela Sombria | Raid | Loot épico | Extremo |

> Zonas perigosas têm recursos mais valiosos mas expõem o jogador a PvP e mobs mais fortes. O risco é a regulação natural da economia.

---

## Core Loop

```
COLETA → CRAFT → COMBATE → PROGRESSÃO → COLETA ...
           ↓                    ↓
        MERCADO ←——————— EQUIPAMENTOS
```

1. **Coleta** — o jogador coleta recursos nas zonas do mundo (madeira, pedra, ervas, minério). Cada coleta tem delay com barra de progresso, simulando o tempo de trabalho.
2. **Craft** — recursos viram itens: ferramentas, armas, armaduras, poções. Tudo craftado por jogadores.
3. **Combate** — jogador usa itens craftados pra combater mobs, ganhar XP global, loot e ouro.
4. **Progressão** — XP global sobe o level. Atividades específicas desbloqueiam nós na árvore de talentos.
5. **Mercado** — excedente de recursos e itens vai pro mercado. Jogadores compram o que não produzem.

---

## Sistema de Progressão

### Level Global

- Toda atividade (coleta, craft, combate) gera XP global.
- Level global vai de 1 a 100.
- Level desbloqueia acesso a zonas de maior perigo e tiers de equipamento.
- Não determina poder direto — apenas acesso.

### Árvore de Talentos

Cada categoria de atividade tem sua própria árvore independente. Nós são desbloqueados ao acumular pontos de atividade específica.

#### Árvores planejadas

**Coleta**
```
Lenhador I → Lenhador II → Mestre Lenhador
    └→ Galhos extras (chance +20%)
    
Minerador I → Minerador II → Mestre Minerador
    └→ Veia dupla (chance de coletar 2x)
    
Herborista I → Herborista II → Mestre Herborista
    └→ Ervas raras (acesso a plantas de zonas perigosas)
```

**Crafting**
```
Ferreiro I → Ferreiro II → Mestre Ferreiro
    └→ Forja perfeita (chance de criar item de tier superior)
    
Alquimista I → Alquimista II → Mestre Alquimista
    └→ Poções concentradas (efeito duplo)
    
Carpinteiro I → Carpinteiro II → Mestre Carpinteiro
    └→ Estruturas (base, bancada avançada)
```

**Combate**
```
Guerreiro I → Guerreiro II → Mestre Guerreiro
    └→ Golpe poderoso (dano crítico)
    
Arqueiro I → Arqueiro II → Mestre Arqueiro
    └→ Tiro certeiro (ignora armadura)
    
Mago I → Mago II → Mestre Mago
    └→ Feitiço amplificado (dano em área)
```

**Regras da árvore:**
- Um jogador pode ter talentos em múltiplas árvores, mas cada uma exige dedicação real de tempo.
- Talentos mais profundos exigem que os anteriores estejam desbloqueados.
- Não existe reset de talentos — suas escolhas refletem sua história no jogo.

---

## Economia

### Princípio fundamental

NPCs não vendem nada de valor. Toda a cadeia de produção — da coleta ao item final — passa por jogadores. O mercado é o reflexo da atividade real do servidor.

### Mercado

- **Mercado global** — jogadores postam itens com preço livre. Oferta e demanda determinam o valor de tudo.
- **Taxa de transação** — uma pequena % vai para um fundo de guild ou manutenção de zona (define-se com o sistema de territórios).
- **Sem preço fixo** — não existe tabelamento. Se Madeira é escassa, o preço sobe. Jogadores que antecipam isso lucram.

### Cadeia de dependência

```
Coletores → Materiais brutos
Artesãos  → Itens processados (tábuas, barras de metal)
Ferreiros → Armas e armaduras
Alquimistas → Poções e consumíveis
Guerreiros → Loot de dungeons/raids → Materiais raros de volta ao mercado
```

### Ouro

- Ouro entra no jogo via: venda de loot de combate a NPCs (sink de loot), drops de dungeons/raids.
- Ouro sai do jogo via: taxas de mercado, custos de guild, compra de itens de qualidade de vida em NPCs.
- Não existe "farm infinito de gold" — a entrada é controlada pelo conteúdo PvE.

---

## Sistema de Combate

> ⚠️ Sistema a ser detalhado em spec própria. Este é o design de intenção.

### Princípios

- Combate usa o mesmo padrão de delay com barra de progresso da coleta — você inicia o ataque, aguarda o resultado.
- Mobs têm HP, nível e loot table definidos no servidor.
- O resultado depende dos atributos do personagem (influenciados pelos itens equipados e talentos).

### Atributos

| Atributo | Influencia |
|----------|-----------|
| Força | Dano físico |
| Resistência | HP máximo, redução de dano |
| Destreza | Chance de acerto, velocidade de ataque |
| Inteligência | Dano mágico, MP |

### Progressão por combate

- Cada mob derrotado gera XP global e XP de combate (alimenta talentos de guerreiro/arqueiro/mago).
- Loot de mobs inclui materiais raros que não existem em coleta — dependência do conteúdo PvE pra economia.

---

## Coleta

Sistema já implementado. Direção futura:

- **Tiers de recursos** — Madeira Comum, Madeira de Lei, Madeira Élfica. Tier mais alto requer nível de talento mais alto e acesso a zonas de maior perigo.
- **Ferramentas influenciam coleta** — um Machado de Ferro coleta mais rápido que as mãos. Um Machado de Aço ainda mais. Equipamento importa até pra coleta.
- **Eventos de coleta** — recursos temporariamente abundantes em zonas específicas, criando corridas de jogadores.

---

## Crafting

Sistema já implementado. Direção futura:

- **Tiers de receitas** — receitas básicas disponíveis para todos. Receitas avançadas dropam de dungeons ou são descobertas por Mestres Artesãos.
- **Qualidade do item** — chance de craft resultar em item Normal, Bom ou Excepcional, influenciada pelo talento de crafting.
- **Bancadas especializadas** — Forja (armas/armaduras), Alquimia (poções), Carpintaria (estruturas). Cada bancada pode ser melhorada por guilds.

---

## Sistemas Planejados

### Guilds

**Visão:** Começa como grupo social com banco compartilhado e bônus coletivos. Ao crescer, pode reivindicar territórios, controlar pontos de coleta e cobrar impostos.

**Progressão de guild:**
- Tier 1 — Social: banco, chat, bônus de XP em grupo
- Tier 2 — Estabelecida: sede própria, bancadas avançadas compartilhadas
- Tier 3 — Territorial: controle de zonas, impostos sobre recursos, guerras de território

### Dungeons

**Visão:** Instâncias privadas para grupos de até 5 jogadores. Três andares com mobs crescentes e um boss final. Drops exclusivos que alimentam a economia.

**Dificuldades:**
- Normal — acessível com equipamento básico
- Heroico — exige coordenação e gear intermediário
- Mítico — exige especialização e gear de raid

### Raids

**Visão:** Eventos de mundo que aparecem em zonas específicas com timer público. Qualquer jogador pode participar; guilds organizadas completam os níveis difíceis.

**Estrutura:**
- Evento anunciado no LOG de todos os jogadores online
- Ondas de mobs culminando em boss de raid
- Recompensas escalonadas por contribuição (dano, cura, suporte)
- Hard mode exige coordenação de guild e abre loot épico exclusivo

### Territórios

**Visão:** Zonas podem ser reivindicadas por guilds. Controlador da zona recebe % dos recursos coletados ali e pode cobrar pedágio. Cria conflito político e econômico orgânico.

---

## Estado Atual — O que já existe

| Sistema | Status |
|---------|--------|
| Conexão WebSocket | ✅ Completo |
| Chat entre jogadores | ✅ Completo |
| LOG de eventos separado | ✅ Completo |
| Inventário com persistência | ✅ Completo |
| Sistema de Craft | ✅ Completo |
| Sistema de Coleta com delay | ✅ Completo |
| Painel de jogadores online | ✅ Completo |
| Auto-reconexão | ✅ Completo |
| Nick único por servidor | ✅ Completo |
| Personagem (estático) | 🔄 Parcial — sem persistência, sem progressão |
| Combate | ❌ Não iniciado |
| Mercado | ❌ Não iniciado |
| Árvore de talentos | ❌ Não iniciado |
| Guilds | ❌ Não iniciado |
| Dungeons | ❌ Não iniciado |
| Raids | ❌ Não iniciado |

---

## Próximos passos recomendados

1. **Persistência do personagem** — salvar level, XP, ouro e talentos por nick (pré-requisito pra tudo)
2. **Combate básico** — mobs, HP, delay de ataque, loot simples
3. **Progressão** — XP global sobe level, atividades geram pontos de talento
4. **Árvore de talentos** — primeiro ramo: coleta (já temos a base)
5. **Mercado** — listagem e compra de itens entre jogadores
6. **Tiers de recursos e receitas** — madeira comum → madeira de lei → madeira élfica

---

*Killian Online é um jogo vivo. Este documento será atualizado conforme o jogo evolui.*
