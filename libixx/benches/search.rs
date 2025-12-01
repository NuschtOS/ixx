use criterion::{Criterion, criterion_group, criterion_main};
use libixx::Index;
use std::{fs::File, hint::black_box};

fn criterion_benchmark(c: &mut Criterion) {
  let mut file = match File::open("../index.ixx") {
    Ok(f) => f,
    Err(e) => {
      eprintln!("index.ixx is missing, you can download one from https://HEAD.nuschtos-search.pages.dev/data/packages/index.ixx and place it in the root of the project: {}", e);
      std::process::exit(1);
    }
  };
  let index = Index::read_from(&mut file).unwrap();

  c.bench_function("search for zoo", |b| {
    b.iter(|| index.search(None, black_box("zoo"), 500))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
