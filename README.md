# Killian Online

Um MMORPG de terminal ambientado no mundo de **Aldenmoor** — onde sua identidade é definida pelo que você faz, não por uma classe escolhida na criação do personagem.

Minere pedras nas cavernas de Greyrock, corte madeira nas florestas de Everfen, combata criaturas nos campos abertos e leve seus recursos ao mercado. Cada ação molda quem você se torna.

```
┌─────────────────┬──────────────────────────────────────┐
│ [1] PERSONAGEM  │                                      │
│ Classe: Aventur │           C H A T                    │
│ Nivel:  1       │                                      │
│ HP:  100/100    │  jogador1: alguem quer madeira?      │
│ MP:   35/ 35    │  jogador2: tenho 20, quanto paga?    │
│ Ouro: 150       │                                      │
├─────────────────┤                                      │
│ [2] INVENTARIO  ├──────────────────────────────────────┤
│ ▶ Madeira  x12  │                                      │
│   Pedra     x6  │           L O G                      │
├─────────────────┤                                      │
│ [3] COLETA      │  Floresta: Madeira x3 coletada       │
│ ▶ Floresta  8s  │  Craft: Espada Longa craftada!       │
│   Mina     10s  │                                      │
├─────────────────┘                                      │
│ [4] CRAFT                                              │
│ [5] ONLINE                                             │
└────────────────────────────────────────────────────────┘
```

## O mundo

**Aldenmoor** é um continente de fantasia medieval onde os jogadores constroem a economia e escrevem a história. Não existe loja de NPC que resolva seus problemas — cada espada foi forjada por alguém, cada poção foi colhida por alguém, cada ouro ganho foi suado.

As zonas do mundo têm recursos, criaturas e histórias próprias. Aventureiros iniciantes encontram campos e florestas seguras. Os mais experientes se aprofundam nas minas e ruínas onde os recursos — e os perigos — são maiores.

## Como jogar

Baixe o cliente para o seu sistema em [Releases](../../releases) e execute no terminal:

```bash
# macOS / Linux
./killian-client

# Windows
killian-client.exe
```

O servidor `wss://killian.spellbook.app.br` já vem preenchido. Digite seu nick, escolha uma senha e entre. **Primeiro acesso cria a conta automaticamente** — suas credenciais são lembradas nos acessos seguintes.

### Controles

| Tecla | Ação |
|-------|------|
| `Tab` | Alterna entre painéis |
| `1` – `5` | Vai direto para o painel |
| `↑` `↓` | Navega lista |
| `Enter` | Executa ação (coletar, craftar) |
| `x` | Cancela coleta em andamento |
| `i` | Modo inserção — digitar no chat |
| `Esc` | Modo normal / sair |
| `Ctrl+C` | Encerra o cliente |

## Rodar servidor localmente

Requer [Rust](https://rustup.rs).

```bash
git clone https://github.com/Durukar/Killian-Project
cd Killian-Project

# Terminal 1 — servidor
cargo run -p killian-server

# Terminal 2 — cliente
cargo run -p killian-client
```

Para expor o servidor publicamente com Cloudflare Tunnel:

```bash
cloudflared tunnel run killian
```

## O que já existe

- Coleta de recursos com barra de progresso por zona
- Crafting com receitas e validação de ingredientes
- Inventário persistido por conta
- Chat entre jogadores em tempo real
- Log de eventos separado do chat
- Autenticação com nick e senha
- Reconexão automática com backoff exponencial
- Múltiplos jogadores simultâneos
- Build automático para Windows, macOS e Linux via GitHub Actions

## Roadmap

- [ ] Mapa do mundo com zonas navegáveis
- [ ] Sistema de combate contra mobs
- [ ] Progressão — XP, level e árvore de talentos por atividade
- [ ] Mercado entre jogadores
- [ ] Guilds e territórios
- [ ] Dungeons e raids

## Licença

MIT
