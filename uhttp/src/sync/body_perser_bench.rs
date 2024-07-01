use divan::bench;
use divan::Bencher;

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

  b.bench_local(move || {
    body_parser::bytes(&mut data)
  })
}

#[bench]
fn bench_utf8(b: Bencher) {
  let data = generate_test_data(DEFAULT_STR_LENGTH);
  let mut data = data.as_bytes();

  b.bench_local(move || {
    body_parser::utf8(&mut data)
  })
}

#[bench]
fn bench_utf8_unchecked(b: Bencher) {
  let data = generate_test_data(DEFAULT_STR_LENGTH);
  let mut data = data.as_bytes();

  b.bench_local(move || {
    unsafe { body_parser::utf8_unchecked(&mut data) }
  })
}
