export class ChatModel {
  constructor() {
    this.nick = "web-player";
    this.server = "ws://127.0.0.1:7001";
    this.connected = false;
    this.lines = ["[system] web client ready"];
  }

  setConnectionConfig({ nick, server }) {
    this.nick = nick;
    this.server = server;
  }

  setConnected(value) {
    this.connected = value;
    this.pushSystem(value ? "connected" : "disconnected");
  }

  pushSystem(text) {
    this.lines.push(`[system] ${text}`);
    this.truncate();
  }

  pushChat(from, text) {
    this.lines.push(`${from}: ${text}`);
    this.truncate();
  }

  truncate() {
    const limit = 500;
    if (this.lines.length > limit) {
      this.lines.splice(0, this.lines.length - limit);
    }
  }
}
