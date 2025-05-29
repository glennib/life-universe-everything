use crate::simulator::Age;
use crate::simulator::Year;
use crate::simulator::run;

mod simulator;

fn main() {
	let sr = run(10_000_000_000, Year(2_000), 1_000, Age(120), 105, 2.06406);
	println!(
		"{:#?}",
		sr.cohort_fertility
			.0
			.iter()
			.map(|(&year, &cd)| { (year, cd.ratio()) })
			.collect::<Vec<_>>()
	);
	println!("initial population: {:>12}", sr.initial_population.count());
	println!("  final population: {:>12}", sr.final_population.count());
}
