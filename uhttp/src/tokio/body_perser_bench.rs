use divan::bench;
use divan::Bencher;
use tokio::runtime::Builder;

use super::body_parser;

const DEFAULT_STR_LENGTH: usize = 100000000;

fn generate_test_data(n: usize) -> String {
  let mut data = String::new();
  for i in 0..n {
    data.push_str(&format!("{}", i))
  }
  data
}

#[bench]
fn bench_bytes(b: Bencher) {
  let data = generate_test_data(DEFAULT_STR_LENGTH);
  let mut data = data.as_bytes();

  let rt = Builder::new_current_thread().build().unwrap();

  b.bench_local(move || {
    rt.block_on(body_parser::bytes(&mut data))
  })
}

#[bench]
fn bench_utf8(b: Bencher) {
  let data = generate_test_data(DEFAULT_STR_LENGTH);
  let mut data = data.as_bytes();

  let rt = Builder::new_current_thread().build().unwrap();

  b.bench_local(move || {
    rt.block_on(body_parser::utf8(&mut data))
  })
}

#[bench]
fn bench_utf8_unchecked(b: Bencher) {
  let data = generate_test_data(DEFAULT_STR_LENGTH);
  let mut data = data.as_bytes();

  let rt = Builder::new_current_thread().build().unwrap();

  b.bench_local(move || {
    rt.block_on(unsafe { body_parser::utf8_unchecked(&mut data) })
  })
}
