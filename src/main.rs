use crate::simulator::Age;
use crate::simulator::Count;
use crate::simulator::Parameters;
use crate::simulator::SimulationResult;
use eframe::Frame;
use eframe::egui;
use eframe::egui::Context;

mod simulator;

fn main() {
	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([400.0, 300.0])
			.with_min_inner_size([300.0, 220.0]),
		..Default::default()
	};
	eframe::run_native(
		"life-universe-everything",
		native_options,
		Box::new(|_cc| Ok(Box::new(MyApp::default()))),
	)
	.unwrap();
}

struct MyApp {
	parameters: Parameters,
	target_total_fertility_rate_default: f64,
}

impl Default for MyApp {
	fn default() -> Self {
		let parameters = Parameters {
			initial_population: 10_000_000_000,
			n_years: 1_000,
			max_age: Age(120),
			males_per_100_females: 105,
			target_total_fertility_rate: 2.06406,
		};
		Self {
			// let sr = run(10_000_000_000, Year(2_000), 1_000, Age(120), 105, 2.06406);
			parameters,
			target_total_fertility_rate_default: parameters.target_total_fertility_rate,
		}
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		egui::TopBottomPanel::top("top").show(ctx, |ui| {
			ui.heading("Life, the Universe and Everything");
			ui.horizontal(|ui| {
				let mut initial_population_exp =
					(self.parameters.initial_population as f64).log10();
				ui.add(
					egui::Slider::new(&mut initial_population_exp, 3.0..=14.0)
						.text("initial population (exp10)"),
				);
				self.parameters.initial_population =
					10.0_f64.powf(initial_population_exp).round() as Count;
				ui.label(format!("{}", self.parameters.initial_population));
			});
			ui.add(egui::Slider::new(&mut self.parameters.n_years, 300..=2000).text("years"));
			ui.add(
				egui::Slider::new(&mut self.parameters.males_per_100_females, 80..=120)
					.text("males per 100 females"),
			);
			ui.horizontal(|ui| {
				ui.add(
					egui::Slider::new(&mut self.parameters.target_total_fertility_rate, 0.0..=3.0)
						.text("target fertility rate"),
				);
				if ui.button("stable?").clicked() {
					let mut integrated = 0.0;
					let mut loops: usize = 0;
					loop {
						if loops > 1_000 {
							println!("failed to converge");
							break;
						}
						let SimulationResult {
							initial_population,
							final_population,
							..
						} = self.parameters.run();
						let loss =
							initial_population.count() as i64 - final_population.count() as i64;
						println!("loss: {loss}");
						if loss.abs() < 10_000 {
							println!("converged!");
							break;
						}
						let loss = loss as f64;
						integrated += loss * 0.000000000001;
						integrated = integrated.clamp(-1.0, 1.0);
						let p = loss * 0.000000000002;
						self.parameters.target_total_fertility_rate =
							(self.target_total_fertility_rate_default + p + integrated).clamp(0.0, 3.0);
						println!(
							"tfr = {:.4} = {:.4} + (p: {:.4}) + (i: {:.4})",
							self.parameters.target_total_fertility_rate,
							self.target_total_fertility_rate_default,
							p,
							integrated,
						);
						loops += 1;
					}
				}
			});
		});

		let SimulationResult {
			initial_population,
			final_population,
			cohort_fertility,
		} = self.parameters.run();

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label(format!(
				"Initial population: {}",
				initial_population.count()
			));
			ui.label(format!("Final population: {}", final_population.count()));
			ui.label(format!("Actual fertility: {:.3}", cohort_fertility.avg()))
		});
	}
}
