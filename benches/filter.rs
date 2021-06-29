use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ycm_core::core::candidate::*;
use ycm_core::core::query::*;

fn generate_candidates_with_common_prefix(prefix: &str, n: usize) -> Vec<String> {
    let mut candidates = Vec::with_capacity(n);

    for i in 0..n {
        let mut candidate = String::new();
        let mut letter = i as u32;
        for _ in 0..5 {
            candidate.insert(
                0,
                char::from_u32((letter % 26) as u32 + b'a' as u32).unwrap(),
            );
            letter /= 26;
        }
        candidate.insert_str(0, prefix);
        candidates.push(candidate);
    }

    candidates
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let q = "aA";
    for n in [1, 16, 256, 4096, 65536] {
        let candidates = generate_candidates_with_common_prefix("a_A_a_", n);
        c.bench_function(&format!("Unstored {}", n), |b| {
            b.iter(|| {
                let candidates = candidates
                    .iter()
                    .map(|s| Candidate::new(&s))
                    .collect::<Vec<_>>();
                let q = Word::new(q);
                let results = filter_and_sort_candidates(&candidates, &q, n);
                black_box(results);
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
