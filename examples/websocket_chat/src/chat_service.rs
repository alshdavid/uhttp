use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

pub struct ChatService {
  messages: Vec<(String, String)>,
  subscriptions: Vec<UnboundedSender<(String, String)>>,
}

impl ChatService {
  pub fn new() -> Self {
    Self {
      messages: Vec::new(),
      subscriptions: Default::default(),
    }
  }

  pub fn subsribe(&mut self) -> UnboundedReceiver<(String, String)> {
    let (tx, rx) = unbounded_channel::<(String, String)>();
    self.subscriptions.push(tx);
    rx
  }

  pub fn get(&self) -> Vec<(String, String)> {
    self.messages.clone()
  }

  pub fn new_message(
    &mut self,
    author: String,
    message: String,
  ) {
    self.messages.push((author.clone(), message.clone()));
    self
      .subscriptions
      .retain(|tx| !tx.send((author.clone(), message.clone())).is_err());
  }
}
