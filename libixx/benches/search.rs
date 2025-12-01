use criterion::{Criterion, criterion_group, criterion_main};
use libixx::Index;
use std::{fs::File, hint::black_box};

fn criterion_benchmark(c: &mut Criterion) {
  let mut file = File::open("../packages/index.ixx").unwrap();
  let index = Index::read_from(&mut file).unwrap();

  c.bench_function("search for zoo", |b| {
    b.iter(|| index.search(None, black_box("zoo"), 500))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
