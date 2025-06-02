use eframe::Frame;
use eframe::egui;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Grid;
use eframe::egui::Hyperlink;
use eframe::egui::ScrollArea;
use eframe::egui::SliderClamping;
use egui_plot::Bar;
use egui_plot::BarChart;
use egui_plot::Plot;

use crate::optimizer::solve;
use crate::simulator::Age;
use crate::simulator::Parameters;
use crate::simulator::SimulationResult;

pub struct MyApp {
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
						ui.add(
							egui::Slider::new(
								&mut self.parameters.initial_population,
								10_u64.pow(3)..=10_u64.pow(13),
							)
							.logarithmic(true),
						);
						ui.label("initial population");
						ui.end_row();
						ui.horizontal(|ui| {
							ui.add(
								egui::Slider::new(&mut self.parameters.n_years, 0..=10_000)
									.integer()
									.clamping(SliderClamping::Edits),
							);
							if ui.button("-").clicked() {
								self.parameters.n_years -= 1;
							}
							if ui.button("+").clicked() {
								self.parameters.n_years += 1;
							}
						});
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
						ui.label("Final population");
						ui.label(format!("{}", self.solution.final_population.count()));
						ui.end_row();
						ui.label("Actual fertility");
						ui.label(format!("{:.3}", self.solution.cohort_fertility.avg()));
						ui.end_row();
					});
			});
		});

		egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
			ui.add(
				Hyperlink::new("https://github.com/glennib/life-universe-everything")
					.open_in_new_tab(true),
			);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ScrollArea::both().show(ui, |ui| {
				ui.group(|ui| {
					ui.heading("Final age distribution");
					Plot::new("final_age_distribution")
						.show_grid([false, false])
						.height(300.0)
						.show(ui, |ui| {
							let fp = &self.solution.final_population;
							let max_age = fp
								.males
								.keys()
								.chain(fp.females.keys())
								.copied()
								.max()
								.unwrap();
							let bars = (0..=max_age.0)
								.map(Age)
								.flat_map(|age| {
									let m = fp.males[&age] as f64;
									let f = fp.females[&age] as f64;
									let age = age.0 as f64;
									let male = Bar::new(age, m).fill(Color32::GREEN);
									let female = Bar::new(age + 0.25, f).fill(Color32::RED);
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
								.iter_mf()
								.flat_map(|(year, (m, f))| {
									[
										Bar::new(year.0 as f64, m as f64).fill(Color32::GREEN),
										Bar::new(year.0 as f64, f as f64)
											.fill(Color32::RED)
											.base_offset(m as f64),
									]
								})
								.collect();
							ui.bar_chart(BarChart::new("bc2", bars));
						});
				});
				#[cfg(not(target_arch = "wasm32"))]
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
		});
	}
}
