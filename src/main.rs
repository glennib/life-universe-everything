use std::collections::BTreeMap;

use eframe::Frame;
use eframe::egui;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Grid;
use egui_plot::Bar;
use egui_plot::BarChart;
use egui_plot::Plot;

use crate::optimizer::solve;
use crate::simulator::Age;
use crate::simulator::Count;
use crate::simulator::Gender;
use crate::simulator::Parameters;
use crate::simulator::SimulationResult;

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
	solution: SimulationResult,
	original_parameters: Parameters,
	out_file: String,
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
		let parameters = solve(parameters);
		let solution = parameters.run();
		Self {
			// let sr = run(10_000_000_000, Year(2_000), 1_000, Age(120), 105, 2.06406);
			parameters,
			solution,
			original_parameters: parameters,
			out_file: String::from("data.json5"),
		}
	}
}

impl eframe::App for MyApp {
	fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
		let prev_params = self.parameters;
		egui::TopBottomPanel::top("top").show(ctx, |ui| {
			ui.heading("Life, the Universe and Everything");
			ui.group(|ui| {
				let grid = Grid::new("grid_settings")
					.num_columns(3)
					.striped(true)
					.show(ui, |ui| {
						let mut initial_population_exp =
							(self.parameters.initial_population as f64).log10();
						ui.add(egui::Slider::new(&mut initial_population_exp, 3.0..=14.0));
						ui.label("initial population (10^x)");
						self.parameters.initial_population =
							10.0_f64.powf(initial_population_exp).round() as Count;
						ui.label(format!("{}", self.parameters.initial_population));
						ui.end_row();

						ui.add(egui::Slider::new(
							&mut self.parameters.n_years,
							1000..=10_000,
						));
						ui.label("years");
						ui.end_row();

						ui.add(egui::Slider::new(
							&mut self.parameters.males_per_100_females,
							80..=120,
						));
						ui.label("males per 100 females");
						ui.end_row();

						ui.add(egui::Slider::new(
							&mut self.parameters.infant_mortality_rate,
							0.001..=0.020,
						));
						ui.label("infant mortality rate");
						ui.end_row();

						ui.add(egui::Slider::new(
							&mut self.parameters.target_total_fertility_rate,
							0.0..=3.0,
						));
						ui.label("target fertility rate");
						if ui.button("stabilize").clicked() {
							let parameters = solve(self.parameters);
							self.parameters = parameters;
						}
						ui.end_row();
					});
				if ui
					.add_sized([grid.response.rect.width(), 0.0], Button::new("reset"))
					.clicked()
				{
					self.parameters = self.original_parameters;
				}
			});

			if prev_params != self.parameters {
				self.solution = self.parameters.run();
			}

			ui.group(|ui| {
				Grid::new("grid_summaries")
					.num_columns(2)
					.striped(true)
					.show(ui, |ui| {
						ui.label("Initial population");
						ui.label(format!("{}", self.solution.initial_population.count()));
						ui.end_row();
						ui.label("Final population");
						ui.label(format!("{}", self.solution.final_population.count()));
						ui.end_row();
						ui.label("Actual fertility");
						ui.label(format!("{:.3}", self.solution.cohort_fertility.avg()));
						ui.end_row();
					});
			});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.group(|ui| {
				ui.heading("Final age distribution");
				Plot::new("qwerty")
					.show_grid([false, false])
					.height(300.0)
					.show(ui, |ui| {
						let bars = self
							.solution
							.final_population
							.0
							.iter()
							.fold(BTreeMap::new(), |mut acc, ((age, gender), &count)| {
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
								let female =
									Bar::new(age.0 as f64, f).base_offset(m).fill(Color32::RED);
								[male, female]
							})
							.collect();
						ui.bar_chart(BarChart::new("bc", bars).name("Age distribution"));
					});
			});
			ui.group(|ui| {
				ui.heading("Total population over time");
				Plot::new("tl")
					.show_grid([false, false])
					.height(200.0)
					.show(ui, |ui| {
						let bars = self
							.solution
							.timeline
							.iter()
							.flat_map(|(year, data)| {
								[
									Bar::new(year.0 as f64, data.males as f64).fill(Color32::GREEN),
									Bar::new(year.0 as f64, data.females as f64)
										.fill(Color32::RED)
										.base_offset(data.males as f64),
								]
							})
							.collect();
						ui.bar_chart(BarChart::new("bc2", bars));
					});
			});
			ui.group(|ui| {
				ui.horizontal(|ui| {
					ui.text_edit_singleline(&mut self.out_file);
					if ui.button("Save").clicked() {
						let s = json5::to_string(&self.solution).unwrap();
						std::fs::write(&self.out_file, &s).unwrap();
						println!("Stored {} bytes to {}", s.len(), self.out_file);
					}
				});
			});
		});
	}
}
