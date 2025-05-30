use crate::optimizer::solve;
use crate::simulator::Age;
use crate::simulator::Count;
use crate::simulator::Gender;
use crate::simulator::Parameters;
use eframe::Frame;
use eframe::egui;
use eframe::egui::Color32;
use eframe::egui::Context;
use egui_plot::Bar;
use egui_plot::BarChart;
use egui_plot::Plot;
use std::collections::BTreeMap;

mod optimizer;
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
}

impl Default for MyApp {
	fn default() -> Self {
		let parameters = Parameters {
			initial_population: 10_000_000_000,
			n_years: 2_000,
			max_age: Age(120),
			males_per_100_females: 105,
			target_total_fertility_rate: 2.06406,
			infant_mortality_rate: 0.005,
		};
		Self {
			// let sr = run(10_000_000_000, Year(2_000), 1_000, Age(120), 105, 2.06406);
			parameters,
		}
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		let sr = egui::TopBottomPanel::top("top")
			.show(ctx, |ui| {
				ui.group(|ui| {
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
					ui.add(
						egui::Slider::new(&mut self.parameters.n_years, 1000..=10_000)
							.text("years"),
					);
					ui.add(
						egui::Slider::new(&mut self.parameters.males_per_100_females, 80..=120)
							.text("males per 100 females"),
					);
					ui.add(
						egui::Slider::new(
							&mut self.parameters.infant_mortality_rate,
							0.001..=0.010,
						)
						.text("infant mortality rate"),
					);
					ui.horizontal(|ui| {
						ui.add(
							egui::Slider::new(
								&mut self.parameters.target_total_fertility_rate,
								0.0..=3.0,
							)
							.text("target fertility rate"),
						);
						if ui.button("stabilize").clicked() {
							let parameters = solve(self.parameters);
							self.parameters = parameters;
						}
					});
				});

				let sr = self.parameters.run();

				ui.group(|ui| {
					ui.label(format!(
						"Initial population: {}",
						sr.initial_population.count()
					));
					ui.label(format!("Final population: {}", sr.final_population.count()));
					ui.label(format!(
						"Actual fertility: {:.3}",
						sr.cohort_fertility.avg()
					))
				});
				sr
			})
			.inner;

		egui::CentralPanel::default().show(ctx, |ui| {
			Plot::new("qwerty").height(300.0).show(ui, |ui| {
				let bars = sr
					.final_population
					.0
					.into_iter()
					.fold(BTreeMap::new(), |mut acc, ((age, gender), count)| {
						let c: &mut (f64, f64) = acc.entry(age).or_default();
						match gender {
							Gender::Male => c.0 += count as f64,
							Gender::Female => c.1 += count as f64,
						}
						acc
					})
					.into_iter()
					.flat_map(|(age, (m, f))| {
						let male = Bar::new(age.0 as f64, m).fill(Color32::GREEN);
						let female = Bar::new(age.0 as f64, f).base_offset(m).fill(Color32::RED);
						[male, female]
					})
					.collect();
				ui.bar_chart(BarChart::new("bc", bars));
			});
			Plot::new("tl").height(200.0).show(ui, |ui| {
				let bars = sr
					.timeline
					.into_iter()
					.map(|(year, count)| Bar::new(year.0 as f64, count as f64))
					.collect();
				ui.bar_chart(BarChart::new("bc2", bars));
			});
		});
	}
}
