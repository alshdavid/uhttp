pub type HandleFunc = Box<
  dyn Send
    + Sync
    + Fn(
      crate::Request,
      crate::Response,
    ) -> std::pin::Pin<Box<dyn Send + std::future::Future<Output = crate::Result<()>>>>,
>;
