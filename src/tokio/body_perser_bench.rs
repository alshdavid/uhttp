use divan::bench;
use divan::Bencher;

#[bench]
fn body_parser(b: Bencher) {
  b.bench_local(move || {})
}
