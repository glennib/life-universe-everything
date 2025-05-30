use argmin::core::CostFunction;
use argmin::core::Error;
use argmin::core::Executor;
use argmin::core::OptimizationResult;
use argmin::solver::neldermead::NelderMead;

use crate::simulator::Parameters;
use crate::simulator::SimulationResult;
use crate::simulator::Year;

impl CostFunction for Parameters {
	type Param = f64;
	type Output = f64;

	fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
		let mut p = *self;
		p.target_total_fertility_rate = param.clamp(0.0, 3.0);
		let SimulationResult { timeline, .. } = p.run();
		let (first_year, max_year) = timeline.year_range();
		let end_year = (first_year.0..=max_year.0)
			.map(Year)
			.find(|year| timeline.sum(*year) <= p.initial_population / 3)
			.unwrap_or(max_year);
		let end_sum = timeline.sum(end_year) as f64;
		let halfway_year = Year((end_year.0 - first_year.0) / 2);
		let halfway_sum = timeline.sum(halfway_year) as f64;
		let years = end_year.0 - halfway_year.0;
		let difference = end_sum - halfway_sum;
		let slope = difference / years as f64;
		// println!(
		// 	"tfr={}, halfway_sum={halfway_sum}, end_sum={end_sum}, difference={difference}, years={years}, slope={slope:e}",
		// 	p.target_total_fertility_rate
		// );
		Ok(slope * slope)
	}
}

pub fn solve(parameters: Parameters) -> Parameters {
	let tfr = parameters.target_total_fertility_rate;
	let solver = NelderMead::new(vec![tfr - 0.05, tfr + 0.05]);
	let res = Executor::new(parameters, solver)
		.configure(|state| state.max_iters(10_000))
		.run()
		.unwrap();
	let OptimizationResult { state, .. } = res;
	let target_tfr = state.best_param.unwrap();
	let mut parameters = parameters;
	parameters.target_total_fertility_rate = target_tfr;
	parameters
}
