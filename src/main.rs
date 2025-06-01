use app::MyApp;
use eframe::egui;
mod app;
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
