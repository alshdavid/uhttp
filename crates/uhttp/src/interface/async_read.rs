use std::io;

pub trait AsyncRead: Send + Sync {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}