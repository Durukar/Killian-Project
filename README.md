# Killian Terminal MMORPG POC

POC em Rust com cliente TUI (Ratatui) e transporte WebSocket.

## O que esta pronto

- Tela inicial para conectar (`nick` + endpoint WebSocket)
- Tela de jogo em layout dividido com paineis:
  - `CHAT`
  - `PERSONAGEM`
  - `INVENTARIO`
- Chat em tempo real via WebSocket
- Build de binario para Windows via GitHub Actions

## Estrutura

- `apps/chat-server`: servidor WebSocket
- `apps/chat-client`: cliente TUI com arquitetura MVVM
- `crates/chat-protocol`: mensagens JSON compartilhadas
- `scripts/build-dist.sh`: build release local
- `scripts/start-server.sh`: inicia servidor pelo binario
- `scripts/play.sh`: abre cliente TUI local

## Rodar local (macOS/Linux)

1. Build dos binarios:

```bash
scripts/build-dist.sh
```

2. Subir servidor:

```bash
KILLIAN_BIND=0.0.0.0:7001 scripts/start-server.sh
```

3. Abrir cliente:

```bash
scripts/play.sh
```

4. Na tela inicial do cliente:

- Nick: seu nome
- Servidor: `ws://127.0.0.1:7001` (mesma maquina) ou `ws://192.168.1.22:7001` (rede)
- `Enter` para conectar

## Teclas

- Tela inicial:
  - `Tab`: alterna campo
  - `Enter`: conectar
  - `Esc`: sair
- Tela do jogo:
  - Digitar + `Enter`: envia mensagem no chat
  - `Esc`: sair

## Distribuir binario para Windows (simples)

Workflow pronto:

- `.github/workflows/windows-client.yml`

Como usar:

1. Suba o repositorio no GitHub.
2. Abra `Actions` > `Build Windows Client`.
3. Clique em `Run workflow`.
4. Baixe o artifact `killian-client-windows`.
5. Entregue o zip ao jogador (`killian-client.exe`).

## Variaveis de ambiente

- Servidor: `KILLIAN_BIND` (fallback: `CHAT_BIND`)
- Cliente endpoint default: `KILLIAN_SERVER` (fallback: `CHAT_SERVER`)
- Cliente nick default: `KILLIAN_NICK` (fallback: `USER`)
