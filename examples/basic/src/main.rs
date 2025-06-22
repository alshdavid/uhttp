#![allow(clippy::unused_io_amount)]
#![feature(async_fn_traits)]

use std::{pin::Pin, process::Output, sync::Arc};

use tokio::{io::AsyncWriteExt, sync::mpsc::unbounded_channel};
use async_fn_traits::AsyncFn0;

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//   uhttp::http1::create_server(|_req, res| {
//     Box::pin(async move {
//       res.write_all(b"hello world\n").await?;
//       Ok(())
//     })
//   })
//   .listen("0.0.0.0:8080")
//   .await?;

//   Ok(())
// }

// pub type HandlerFuncResult = anyhow::Result<()>;

// pub trait HandlerFunc:
//   'static
//   + Send
//   + Sync
//   + Copy
//   + for<'a> Fn() -> Pin<Box<dyn Future<Output = HandlerFuncResult> + Send + 'a>>
// {
// }

// impl<F> HandlerFunc for F where
//   F: 'static
//     + Send
//     + Sync
//     + Copy
//     + for<'a> Fn() -> Pin<Box<dyn Future<Output = HandlerFuncResult> + Send + 'a>>
// {
// }


fn handler() {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let value = 42;

  use_async_func(async move || {
    println!("{}", value);
    Ok(())
  }).await?;

  Ok(())
}

// async fn use_async_func<Func>(func: Func) -> anyhow::Result<()>
// where
//     Func: Send + 'static + AsyncFn() -> anyhow::Result<()>,
//     for<'a> Func::CallRefFuture<'a>: Send,
// {
//     tokio::task::spawn(async move { func().await })
//         .await
//         .unwrap()
// }

async fn use_async_func<Func>(func: Func)-> anyhow::Result<()> 
  where 
    Func: Send + 'static + AsyncFn() -> anyhow::Result<()>,
{
  tokio::task::spawn(async move { 
    let fut = func();
    let fut_ptr = Box::new(Box::pin(fut));
    let fut_raw = Box::into_raw(fut_ptr) as *mut Box<Pin<Box<dyn Send + Future<Output=anyhow::Result<()>>>>>;
    let fut = unsafe { Box::from_raw(fut_raw) };
    let fut_raw = fut.await;
    Ok::<(), anyhow::Error>(())
  }).await?
}

// async fn use_async_func<Func, Fut>(func: Func)-> anyhow::Result<()> 
//   where 
//     Func: Send + 'static + AsyncFn0<OutputFuture = Fut>,
//     Fut: Future<Output = anyhow::Result<()>> + Send + Sync + 'static,
//   {
//   tokio::task::spawn(async move { func().await }).await?
// }



// async fn foo<Func, Fut>(func: Func)-> anyhow::Result<()> 
//   where 
//     Func: Send + 'static + AsyncFn0<Output = anyhow::Result<()>, OutputFuture = Fut>,
//     Fut: Future + Send + Sync + 'static
//   {
//   // tokio::task::spawn(async move {
//     let f = func();
//     f.await;
//   // }).await?;

//   println!("lol");
//   Ok(())
// }