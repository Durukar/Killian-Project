export class ChatViewModel {
  static fromModel(model) {
    return {
      nick: model.nick,
      server: model.server,
      status: model.connected ? "ONLINE" : "OFFLINE",
      lines: model.lines,
    };
  }
}
