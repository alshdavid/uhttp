use futures::SinkExt;
use futures::StreamExt;
use http_body_util::BodyExt;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message as WebsocketMessage;
use tokio_tungstenite::tungstenite::protocol::Role;
pub use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame as WebsocketCloseFrame;
pub use tokio_tungstenite::tungstenite::protocol::frame::Frame as WebsocketFrame;
pub use tokio_tungstenite::tungstenite::protocol::frame::FrameHeader as WebsocketFrameHeader;
pub use tokio_tungstenite::tungstenite::protocol::frame::FrameSocket as WebsocketFrameSocket;
pub use tokio_tungstenite::tungstenite::protocol::frame::Utf8Bytes as WebsocketUtf8Bytes;

use crate::ResponseState;

pub struct WebSocket {}

impl WebSocket {
  pub async fn upgrade(
    mut req: crate::Request,
    res: crate::Response,
  ) -> crate::Result<(WebsocketSender, WebsocketReciever)> {
    let mut guard = res.state.write().await;

    let (builder, tx_res) = match std::mem::replace(&mut *guard, ResponseState::Pending) {
      ResponseState::Builder(builder) => builder,
      _ => return Err(crate::Error::generic("Cannot upgrade")),
    };
    drop(guard);

    let on_upgrade = req
      .extensions
      .remove::<hyper::upgrade::OnUpgrade>()
      .ok_or_else(|| crate::Error::generic("Not upgradable"))?;

    let key = req
      .headers()
      .get("Sec-WebSocket-Key")
      .unwrap()
      .to_str()
      .unwrap();
    let accept = derive_accept_key(key);

    let http_res = builder
      .status(101)
      .header("Upgrade", "websocket")
      .header("Connection", "upgrade")
      .header("Sec-WebSocket-Accept", accept)
      .body(
        Empty::<Bytes>::new()
          .map_err(|never| match never {})
          .boxed(),
      )
      .map_err(|e| crate::Error::generic(e.to_string()))?;

    tx_res
      .send(http_res)
      .map_err(|_| crate::Error::generic("Failed to send 101"))?;

    let mut guard = res.state.write().await;
    drop(std::mem::replace(&mut *guard, ResponseState::Done));
    drop(guard);

    let upgraded = on_upgrade.await.expect("Upgrade failed");
    let socket = WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Server, None).await;
    let (socket_send, socket_receive) = socket.split();

    Ok((
      WebsocketSender(socket_send),
      WebsocketReciever(socket_receive),
    ))
  }
}

pub struct WebsocketSender(
  futures::prelude::stream::SplitSink<
    WebSocketStream<TokioIo<hyper::upgrade::Upgraded>>,
    WebsocketMessage,
  >,
);

impl WebsocketSender {
  pub async fn send(
    &mut self,
    msg: WebsocketMessage,
  ) -> crate::Result<()> {
    if self.0.send(msg).await.is_err() {
      return Err(crate::Error::generic("Cannot write to websocket"));
    };
    Ok(())
  }

  pub async fn send_text(
    &mut self,
    text: impl AsRef<str>,
  ) -> crate::Result<()> {
    self
      .send(WebsocketMessage::Text(text.as_ref().into()))
      .await
  }

  pub async fn send_binary(
    &mut self,
    bytes: Vec<u8>,
  ) -> crate::Result<()> {
    self.send(WebsocketMessage::Binary(bytes.into())).await
  }
}

pub struct WebsocketReciever(
  futures::prelude::stream::SplitStream<WebSocketStream<TokioIo<hyper::upgrade::Upgraded>>>,
);

impl WebsocketReciever {
  pub async fn next(&mut self) -> Option<crate::Result<WebsocketMessage>> {
    match self.0.next().await {
      Some(Ok(v)) => Some(Ok(v)),
      Some(Err(err)) => Some(Err(crate::Error::generic(format!("{:?}", err)))),
      None => None,
    }
  }

  pub async fn next_text(&mut self) -> Option<crate::Result<String>> {
    match self.0.next().await {
      Some(Ok(WebsocketMessage::Text(text))) => Some(Ok(text.to_string())),
      None => None,
      _ => Some(Err(crate::Error::generic(
        "Expected websocket message as text",
      ))),
    }
  }
}

fn derive_accept_key(client_key: &str) -> String {
  use base64::Engine as _;
  use base64::engine::general_purpose;
  use sha1::Digest;
  use sha1::Sha1;

  let mut hasher = Sha1::new();
  hasher.update(client_key.as_bytes());
  hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
  general_purpose::STANDARD.encode(hasher.finalize())
}
