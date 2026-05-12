use anyhow::Context;
use killian_protocol::{ClientMsg, ServerMsg};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct NetHandle {
    pub tx: mpsc::UnboundedSender<ClientMsg>,
    pub rx: mpsc::UnboundedReceiver<ServerMsg>,
}

pub async fn connect(endpoint: &str, nick: String, password: String) -> anyhow::Result<NetHandle> {
    let ws_url = normalize_ws_url(endpoint);

    let (ws_stream, _) = connect_async(ws_url.as_str())
        .await
        .with_context(|| format!("falha ao conectar no websocket {ws_url}"))?;

    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<ClientMsg>();
    let (in_tx, in_rx) = mpsc::unbounded_channel::<ServerMsg>();

    let join_payload = serde_json::to_string(&ClientMsg::Join { nick, password })?;
    ws_writer.send(Message::Text(join_payload.into())).await?;

    tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if let Ok(payload) = serde_json::to_string(&msg) {
                if ws_writer.send(Message::Text(payload.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(incoming) = ws_reader.next().await {
            match incoming {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<ServerMsg>(&text) {
                        let _ = in_tx.send(msg);
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }

        let _ = in_tx.send(ServerMsg::System {
            text: "conexao encerrada".to_string(),
        });
    });

    Ok(NetHandle {
        tx: out_tx,
        rx: in_rx,
    })
}

fn normalize_ws_url(endpoint: &str) -> String {
    if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
        endpoint.to_string()
    } else {
        format!("ws://{endpoint}")
    }
}
