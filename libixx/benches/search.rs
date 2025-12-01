use criterion::{Criterion, criterion_group, criterion_main};
use libixx::Index;
use std::{fs::File, hint::black_box};

fn criterion_benchmark(c: &mut Criterion) {
  let mut file = match File::open("../index.ixx") {
    Ok(f) => f,
    Err(e) => {
      eprintln!(
        "index.ixx is missing, you can download one from https://HEAD.nuschtos-search.pages.dev/data/packages/index.ixx and place it in the root of the project: {}",
        e
      );
      std::process::exit(1);
    }
  };
  let index = Index::read_from(&mut file).unwrap();

  c.bench_function("search for hello", |b| {
    b.iter(|| index.search(None, black_box("hello"), 500))
  });

  c.bench_function("search for zoo", |b| {
    b.iter(|| index.search(None, black_box("zoo"), 500))
  });

  c.bench_function("search for python313Packages.cryptography", |b| {
    b.iter(|| index.search(None, black_box("python313Packages.cryptography"), 500))
  });

  c.bench_function("search for python3*.crypto*", |b| {
    b.iter(|| index.search(None, black_box("python313Packages.cryptography"), 500))
  });

  c.bench_function(
    "search for haskell.packages.ghc9103.Facebook-Password-Hacker-Online-Latest-Version
",
    |b| {
      b.iter(|| {
        index.search(
          None,
          black_box(
            "haskell.packages.ghc9103.Facebook-Password-Hacker-Online-Latest-Version
",
          ),
          500,
        )
      })
    },
  );

  c.bench_function(
    "search for haskell.packages.ghc*.Facebook-*-Version
",
    |b| {
      b.iter(|| {
        index.search(
          None,
          black_box(
            "haskell.packages.ghc9103.Facebook-Password-Hacker-Online-Latest-Version
",
          ),
          500,
        )
      })
    },
  );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
