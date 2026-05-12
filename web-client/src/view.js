export class ChatView {
  constructor() {
    this.terminalNode = document.getElementById("terminal");
    this.nickInput = document.getElementById("nickInput");
    this.wsInput = document.getElementById("wsInput");
    this.connectBtn = document.getElementById("connectBtn");
    this.messageInput = document.getElementById("messageInput");
    this.sendBtn = document.getElementById("sendBtn");

    this.terminal = new window.Terminal({
      cursorBlink: true,
      convertEol: true,
      theme: {
        background: "#0b1522",
        foreground: "#d6deeb",
        cyan: "#25c18a",
      },
    });
    this.terminal.open(this.terminalNode);
    this.terminal.writeln("Killian Web Client");
    this.terminal.writeln("-----------------");
  }

  bindConnect(handler) {
    this.connectBtn.addEventListener("click", () => {
      handler({
        nick: this.nickInput.value.trim(),
        server: this.wsInput.value.trim(),
      });
    });
  }

  bindSend(handler) {
    this.sendBtn.addEventListener("click", () => {
      handler(this.messageInput.value);
      this.messageInput.value = "";
    });

    this.messageInput.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        handler(this.messageInput.value);
        this.messageInput.value = "";
      }
    });
  }

  render(vm) {
    this.nickInput.value = vm.nick;
    this.wsInput.value = vm.server;
    this.connectBtn.textContent = vm.status === "ONLINE" ? "Reconnect" : "Connect";

    this.terminal.reset();
    this.terminal.writeln(`Status: ${vm.status}`);
    this.terminal.writeln(`Nick: ${vm.nick}`);
    this.terminal.writeln(`Server: ${vm.server}`);
    this.terminal.writeln("-----------------");

    for (const line of vm.lines) {
      this.terminal.writeln(line);
    }
  }
}
