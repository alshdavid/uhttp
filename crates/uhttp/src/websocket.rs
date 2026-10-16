use std::future::Future;
use std::sync::Arc;
use std::sync::Weak;
use std::time::Duration;

use anyhow::anyhow;
use futures::SinkExt;
use futures::StreamExt;
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use http::header;
use hyper::upgrade::OnUpgrade;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
#[cfg(feature = "json")]
use serde::Serialize;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
pub use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
pub use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::Role;

type WebsocketIo = WebSocketStream<TokioIo<Upgraded>>;
type WebsocketSink = SplitSink<WebsocketIo, Message>;

/// How often to ping the peer. Browsers cannot send pings themselves, so the
/// server has to keep the connection warm or intermediate proxies (which
/// commonly idle out at 60s) will drop it.
const PING_INTERVAL: Duration = Duration::from_secs(30);

pub struct Websocket {
  stream: WebsocketIo,
}

impl Websocket {
  pub fn upgrade<F, Fut>(
    mut req: crate::Request,
    res: crate::Response,
    callback: F,
  ) -> crate::Result<crate::HandlerResponse>
  where
    F: 'static + Send + FnOnce(WebsocketSender, WebsocketReceiver) -> Fut,
    Fut: 'static + Send + Future<Output = crate::Result<()>>,
  {
    let version = req.headers.get(header::SEC_WEBSOCKET_VERSION);

    if version.map(|version| version.as_bytes()) != Some(b"13") {
      return Err(anyhow!("unsupported websocket version"));
    }

    let key = req
      .headers
      .get(header::SEC_WEBSOCKET_KEY)
      .ok_or_else(|| anyhow!("missing websocket key"))?;

    let accept = derive_accept_key(key.as_bytes());

    let on_upgrade = req
      .extensions
      .remove::<OnUpgrade>()
      .ok_or_else(|| anyhow!("connection cannot be upgraded"))?;

    tokio::spawn(async move {
      let Ok(upgraded) = on_upgrade.await else {
        return;
      };

      let stream =
        WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Server, None).await;

      let (sender, receiver) = Websocket { stream }.split();

      tokio::spawn(keepalive(Arc::downgrade(&sender.0)));

      let _ = callback(sender, receiver).await;
    });

    res
      .status(crate::StatusCode::SWITCHING_PROTOCOLS)
      .header(header::CONNECTION, "Upgrade")
      .header(header::UPGRADE, "websocket")
      .header(header::SEC_WEBSOCKET_ACCEPT, accept)
      .body("")
  }

  pub async fn send(
    &mut self,
    message: impl Into<Message>,
  ) -> crate::Result<()> {
    self.stream.send(message.into()).await?;
    Ok(())
  }

  pub async fn recv(&mut self) -> crate::Result<Option<Message>> {
    match self.stream.next().await {
      Some(message) => Ok(Some(message?)),
      None => Ok(None),
    }
  }

  pub async fn close(
    &mut self,
    frame: Option<CloseFrame>,
  ) -> crate::Result<()> {
    self.stream.close(frame).await?;
    Ok(())
  }

  pub fn split(self) -> (WebsocketSender, WebsocketReceiver) {
    let (sender, receiver) = self.stream.split();
    (
      WebsocketSender(Arc::new(Mutex::new(sender))),
      WebsocketReceiver(receiver),
    )
  }

  pub fn reunite(
    sender: WebsocketSender,
    receiver: WebsocketReceiver,
  ) -> crate::Result<Self> {
    let sender = Arc::try_unwrap(sender.0)
      .map_err(|_| anyhow!("websocket sender is still cloned elsewhere"))?
      .into_inner();

    let stream = sender.reunite(receiver.0)?;
    Ok(Self { stream })
  }
}

/// Pings the peer on an interval until the socket errors or every
/// [`WebsocketSender`] has been dropped. Holds a [`Weak`] so it neither keeps the
/// socket alive on its own nor blocks [`Websocket::reunite`].
async fn keepalive(sink: Weak<Mutex<WebsocketSink>>) {
  let mut interval = tokio::time::interval(PING_INTERVAL);
  interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
  interval.tick().await;

  loop {
    interval.tick().await;

    let Some(sink) = sink.upgrade() else {
      return;
    };

    if sink
      .lock()
      .await
      .send(Message::Ping(Default::default()))
      .await
      .is_err()
    {
      return;
    }
  }
}

/// A cloneable handle for sending messages on a websocket.
///
/// Cloning is cheap and every clone shares the same underlying sink, so the
/// connection can be driven from several tasks at once (e.g. one task relaying
/// messages received from a broadcast channel while another handles pongs).
#[derive(Clone)]
pub struct WebsocketSender(Arc<Mutex<WebsocketSink>>);

impl WebsocketSender {
  /// Serialize `message` as JSON and send it as a text frame.
  #[cfg(feature = "json")]
  pub async fn send<S: Serialize>(
    &self,
    message: S,
  ) -> crate::Result<()> {
    let json = serde_json::to_string(&message)?;
    self.send_raw(Message::text(json)).await
  }

  /// Send a raw websocket message.
  pub async fn send_raw(
    &self,
    message: impl Into<Message>,
  ) -> crate::Result<()> {
    self.0.lock().await.send(message.into()).await?;
    Ok(())
  }

  pub async fn close(
    &self,
    frame: Option<CloseFrame>,
  ) -> crate::Result<()> {
    self.0.lock().await.send(Message::Close(frame)).await?;
    Ok(())
  }
}

pub struct WebsocketReceiver(SplitStream<WebsocketIo>);

impl WebsocketReceiver {
  /// Receive the next message, deserializing JSON text/binary frames.
  ///
  /// Ping, Pong and Frame messages are skipped. A close frame (or the socket
  /// ending) yields `Ok(None)`.
  #[cfg(feature = "json")]
  pub async fn recv<D: DeserializeOwned>(&mut self) -> crate::Result<Option<D>> {
    while let Some(message) = self.0.next().await {
      let payload = match message? {
        Message::Text(text) => serde_json::from_str(&text)?,
        Message::Binary(bytes) => serde_json::from_slice(&bytes)?,
        Message::Close(_) => return Ok(None),
        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
      };
      return Ok(Some(payload));
    }
    Ok(None)
  }

  /// Receive the next raw websocket message.
  pub async fn recv_raw(&mut self) -> crate::Result<Option<Message>> {
    match self.0.next().await {
      Some(message) => Ok(Some(message?)),
      None => Ok(None),
    }
  }
}
