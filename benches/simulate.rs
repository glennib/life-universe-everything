use std::hint::black_box;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use life_universe_everything::simulator::Age;
use life_universe_everything::simulator::Parameters;

pub fn benchmark(c: &mut Criterion) {
	let parameters = black_box(Parameters {
		initial_population: 10_000_000_000,
		n_years: 10_000,
		max_age: Age(120),
		males_per_100_females: 105,
		target_total_fertility_rate: 2.0802,
		infant_mortality_rate: 0.0050,
	});
	c.bench_function("simulate", |b| {
		b.iter(|| {
			let _sr = black_box(parameters.run());
		});
	});
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
