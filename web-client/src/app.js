import { ChatModel } from "./model.js";
import { ChatViewModel } from "./viewmodel.js";
import { ChatView } from "./view.js";

const model = new ChatModel();
const view = new ChatView();

let socket = null;

function render() {
  const vm = ChatViewModel.fromModel(model);
  view.render(vm);
}

function connect({ nick, server }) {
  if (!nick || !server) {
    model.pushSystem("nick and server are required");
    render();
    return;
  }

  if (socket) {
    socket.close();
  }

  model.setConnectionConfig({ nick, server });
  model.pushSystem(`connecting to ${server}`);
  render();

  socket = new WebSocket(server);

  socket.addEventListener("open", () => {
    model.setConnected(true);
    socket.send(
      JSON.stringify({
        type: "join",
        nick: model.nick,
      }),
    );
    render();
  });

  socket.addEventListener("close", () => {
    model.setConnected(false);
    render();
  });

  socket.addEventListener("error", () => {
    model.pushSystem("websocket error");
    render();
  });

  socket.addEventListener("message", (event) => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.type === "system") {
        model.pushSystem(payload.text ?? "system message");
      } else if (payload.type === "chat" && payload.line) {
        model.pushChat(payload.line.from, payload.line.text);
      }
      render();
    } catch {
      model.pushSystem("invalid server payload");
      render();
    }
  });
}

function sendMessage(text) {
  const content = text.trim();
  if (!content) {
    return;
  }

  if (!socket || socket.readyState !== WebSocket.OPEN) {
    model.pushSystem("not connected");
    render();
    return;
  }

  socket.send(
    JSON.stringify({
      type: "chat",
      text: content,
    }),
  );
}

view.bindConnect(connect);
view.bindSend(sendMessage);
render();
