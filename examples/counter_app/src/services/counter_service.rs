use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;

use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

pub struct CounterService {
  value: AtomicIsize,
  subscriptions: Mutex<Vec<UnboundedSender<()>>>,
}

impl CounterService {
  pub fn new() -> Self {
    Self {
      value: AtomicIsize::new(0),
      subscriptions: Default::default(),
    }
  }

  pub async fn subsribe(&self) -> UnboundedReceiver<()> {
    let (tx, rx) = unbounded_channel::<()>();
    self.subscriptions.lock().await.push(tx);
    rx
  }

  pub fn get(&self) -> isize {
    self.value.load(Ordering::Relaxed)
  }

  pub async fn increment(&self) {
    self.value.fetch_add(1, Ordering::Relaxed);
    let mut senders = self.subscriptions.lock().await;
    senders.retain(|tx| tx.send(()).is_ok());
  }

  pub async fn decrement(&self) {
    self.value.fetch_sub(1, Ordering::Relaxed);
    let mut senders = self.subscriptions.lock().await;
    senders.retain(|tx| tx.send(()).is_ok());
  }
}
