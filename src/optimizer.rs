use crate::simulator::Parameters;
use crate::simulator::SimulationResult;
use argmin::core::CostFunction;
use argmin::core::Error;
use argmin::core::Executor;
use argmin::core::OptimizationResult;
use argmin::solver::neldermead::NelderMead;

impl CostFunction for Parameters {
	type Param = f64;
	type Output = f64;

	fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
		let mut p = *self;
		p.target_total_fertility_rate = param.clamp(0.0, 3.0);
		let SimulationResult {
			initial_population,
			final_population,
			..
		} = p.run();
		let loss = initial_population.count() as f64 - final_population.count() as f64;
		Ok(loss.abs())
	}
}

pub fn solve(parameters: Parameters) -> Parameters {
	let solver = NelderMead::new(vec![1.9, 2.1]);
	let res = Executor::new(parameters, solver)
		.configure(|state| state.max_iters(10_000))
		.run()
		.unwrap();
	let OptimizationResult {
		problem,
		solver,
		state,
	} = res;
	let target_tfr = state.best_param.unwrap();
	let mut parameters = parameters;
	parameters.target_total_fertility_rate = target_tfr;
	parameters
}
