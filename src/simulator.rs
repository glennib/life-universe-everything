use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Year(pub i32);
impl std::fmt::Display for Year {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Age(pub u8);
impl std::fmt::Display for Age {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct AgeGenderMap {
	pub males: HashMap<Age, Count>,
	pub females: HashMap<Age, Count>,
}

impl AgeGenderMap {
	pub fn count(&self) -> Count {
		self.males.values().sum::<Count>() + self.females.values().sum::<Count>()
	}

	#[allow(unused)]
	pub fn count_gender(&self, gender: Gender) -> Count {
		match gender {
			Gender::Male => self.males.values().sum(),
			Gender::Female => self.females.values().sum(),
		}
	}

	#[allow(unused)]
	pub fn count_age(&self, age: Age) -> Count {
		self.males.get(&age).copied().unwrap_or_default()
			+ self.females.get(&age).copied().unwrap_or_default()
	}

	#[allow(unused)]
	pub fn count_age_gender(&self, age: Age, gender: Gender) -> Count {
		match gender {
			Gender::Male => self.males.get(&age).copied().unwrap_or_default(),
			Gender::Female => self.females.get(&age).copied().unwrap_or_default(),
		}
	}
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct CohortFertility(pub BTreeMap<Year, CohortData>);

impl CohortFertility {
	pub fn avg(&self) -> f64 {
		let sum: f64 = self.0.values().map(|cd| cd.ratio()).sum();
		let count = self.0.len() as f64;
		sum / count
	}
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CohortData {
	#[serde(rename = "f")]
	pub females: Count,
	#[serde(rename = "b")]
	pub births: Count,
}

impl CohortData {
	pub fn ratio(self) -> f64 {
		self.births as f64 / self.females as f64
	}
}

pub type Count = u64;

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug, Hash, Serialize)]
pub enum Gender {
	Male,
	Female,
}

#[derive(Serialize)]
pub struct SimulationResult {
	pub initial_population: AgeGenderMap,
	pub final_population: AgeGenderMap,
	pub cohort_fertility: CohortFertility,
	pub timeline: Timeline,
}

#[derive(Serialize, Default)]
pub struct Timeline {
	pub males: BTreeMap<Year, Count>,
	pub females: BTreeMap<Year, Count>,
}

impl Timeline {
	pub fn insert(&mut self, year: Year, males: Count, females: Count) {
		self.males.insert(year, males);
		self.females.insert(year, females);
	}
	pub fn sum(&self, year: Year) -> Count {
		let (m, f) = self.get_mf(year);
		m + f
	}

	pub fn year_range(&self) -> (Year, Year) {
		let it = self.males.keys().chain(self.females.keys());
		(*it.clone().min().unwrap(), *it.max().unwrap())
	}

	pub fn get_mf(&self, year: Year) -> (Count, Count) {
		(
			self.males.get(&year).copied().unwrap_or_default(),
			self.females.get(&year).copied().unwrap_or_default(),
		)
	}

	pub fn iter_mf(&self) -> impl Iterator<Item = (Year, (Count, Count))> {
		let (year_from, year_to) = self.year_range();
		(year_from.0..=year_to.0)
			.map(Year)
			.map(|year| (year, (self.males[&year], self.females[&year])))
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameters {
	pub initial_population: Count,
	pub n_years: u16,
	pub max_age: Age,
	pub males_per_100_females: u8,
	pub target_total_fertility_rate: f64,
	pub infant_mortality_rate: f64,
}

impl Parameters {
	pub fn run(self) -> SimulationResult {
		let initial_year = Year(0);
		let mut population = PopulationSimulator::new(self);
		let initial_population = population.map.clone();
		let mut timeline = Timeline::default();
		timeline.insert(
			initial_year,
			population.map.count_gender(Gender::Male),
			population.map.count_gender(Gender::Female),
		);

		for year in 0..self.n_years {
			let year = Year(initial_year.0 + i32::from(year));
			population.propagate_age();
			population.handle_births(year);
			population.handle_deaths();
			timeline.insert(
				Year(year.0 + 1),
				population.map.count_gender(Gender::Male),
				population.map.count_gender(Gender::Female),
			);
		}

		let final_population = population.map;

		let mut cohort_fertility = population.cohort_fertility;
		cohort_fertility.0.retain(|&year, _| {
			year.0 >= initial_year.0 + 100 && year.0 <= (initial_year.0 + self.n_years as i32 - 100)
		});

		SimulationResult {
			initial_population,
			final_population,
			cohort_fertility,
			timeline,
		}
	}
}

struct PopulationSimulator {
	map: AgeGenderMap,
	cohort_fertility: CohortFertility,
	parameters: Parameters,
	male_birth_bias: f64,
}

impl PopulationSimulator {
	fn new(parameters: Parameters) -> Self {
		fn age_relative_frequency(age: Age, max_age: Age) -> f64 {
			if age > max_age {
				return 0.0;
			}
			match age.0 {
				0..=14 => 0.25 / 15.0,  // ~1.67% per year
				15..=24 => 0.16 / 10.0, // 1.6% per year
				25..=54 => 0.41 / 30.0, // ~1.37% per year
				55..=64 => 0.09 / 10.0, // 0.9% per year
				65.. => 0.09 / 56.0,    // ~0.16% per year
			}
		}
		let map: HashMap<_, _> = (0..=(parameters.max_age.0 + 1))
			.map(|age| {
				let age = Age(age);
				let rel_freq = age_relative_frequency(age, parameters.max_age);
				let count_each_gender =
					((rel_freq * (parameters.initial_population as f64)) * 0.5) as Count;
				(age, count_each_gender)
			})
			.collect();
		let map = AgeGenderMap {
			males: map.clone(),
			females: map,
		};
		Self {
			cohort_fertility: CohortFertility::default(),
			map,
			parameters,
			male_birth_bias: parameters.males_per_100_females as f64
				/ (parameters.males_per_100_females + 100) as f64,
		}
	}

	fn handle_births(&mut self, year: Year) {
		const fn birth_probability_one_year_nominal(age: u8) -> f64 {
			match age {
				15..=19 => 0.04,
				20..=24 => 0.10,
				25..=29 => 0.13,
				30..=34 => 0.12,
				35..=39 => 0.08,
				40..=44 => 0.03,
				45..=49 => 0.005,
				_ => 0.0,
			}
		}
		const TFR_NOMINAL: f64 = {
			let mut s = 0.0;
			let mut age = 0;
			loop {
				s += birth_probability_one_year_nominal(age);
				if age == 255 {
					break;
				}
				age += 1;
			}
			s
		};
		fn birth_probability_one_year(age: u8, target_tfr: f64) -> f64 {
			birth_probability_one_year_nominal(age) * target_tfr / TFR_NOMINAL
		}

		let newborns = self
			.map
			.females
			.iter()
			.map(|(&age, &females)| {
				(
					age,
					(birth_probability_one_year(age.0, self.parameters.target_total_fertility_rate)
						* (females as f64))
						.round() as Count,
				)
			})
			.inspect(|&(age, births)| {
				let mothers_birth_year = Year(year.0 - age.0 as i32);
				if births > 0 {
					let cf = self
						.cohort_fertility
						.0
						.entry(mothers_birth_year)
						.or_default();
					cf.births += births;
				}
			})
			.map(|(_age, births)| births)
			.sum::<Count>();
		let males = (newborns as f64 * self.male_birth_bias).round() as Count;
		let females = newborns - males;
		let cf = self.cohort_fertility.0.entry(year).or_default();
		cf.females += females;
		*self.map.females.get_mut(&Age(0)).unwrap() += females;
		*self.map.males.get_mut(&Age(0)).unwrap() += males;
	}

	fn handle_deaths(&mut self) {
		fn death_probability_one_year(
			age: Age,
			gender: Gender,
			max_age: Age,
			infant_mortality_rate: f64,
		) -> f64 {
			if age >= max_age {
				return 1.0;
			}
			match gender {
				Gender::Male => match age.0 {
					0 => infant_mortality_rate,
					1 => 0.00039,
					2..=4 => 0.00020,
					5..=9 => 0.00013,
					10..=14 => 0.00010,
					15..=19 => 0.00022,
					20..=24 => 0.00074,
					25..=29 => 0.00097,
					30..=34 => 0.00107,
					35..=39 => 0.00127,
					40..=44 => 0.00174,
					45..=49 => 0.00261,
					50..=54 => 0.00422,
					55..=59 => 0.00689,
					60..=64 => 0.01135,
					65..=69 => 0.01871,
					70..=74 => 0.03066,
					75..=79 => 0.05027,
					80..=84 => 0.08096,
					85..=89 => 0.13257,
					90..=94 => 0.20755,
					95..=99 => 0.31234,
					100.. => 0.43622,
				},
				Gender::Female => match age.0 {
					0 => infant_mortality_rate,
					1 => 0.00030,
					2..=4 => 0.00015,
					5..=9 => 0.00010,
					10..=14 => 0.00008,
					15..=19 => 0.00018,
					20..=24 => 0.00060,
					25..=29 => 0.00080,
					30..=34 => 0.00090,
					35..=39 => 0.00110,
					40..=44 => 0.00150,
					45..=49 => 0.00220,
					50..=54 => 0.00350,
					55..=59 => 0.00570,
					60..=64 => 0.00940,
					65..=69 => 0.01550,
					70..=74 => 0.02540,
					75..=79 => 0.04160,
					80..=84 => 0.06700,
					85..=89 => 0.10970,
					90..=94 => 0.17100,
					95..=99 => 0.25500,
					100.. => 0.36000,
				},
			}
		}

		for (age, count) in &mut self.map.males {
			let probability = death_probability_one_year(
				*age,
				Gender::Male,
				self.parameters.max_age,
				self.parameters.infant_mortality_rate,
			);
			let deaths = (*count as f64 * probability).round();
			*count -= deaths as Count;
		}

		for (age, count) in &mut self.map.females {
			let probability = death_probability_one_year(
				*age,
				Gender::Female,
				self.parameters.max_age,
				self.parameters.infant_mortality_rate,
			);
			let deaths = (*count as f64 * probability).round();
			*count -= deaths as Count;
		}
	}

	fn propagate_age(&mut self) {
		for age in (0..=self.parameters.max_age.0).rev() {
			let [old_male, young_male] = self
				.map
				.males
				.get_disjoint_mut([&Age(age + 1), &Age(age)])
				.map(Option::unwrap);
			let [old_female, young_female] = self
				.map
				.females
				.get_disjoint_mut([&Age(age + 1), &Age(age)])
				.map(Option::unwrap);
			*old_male = *young_male;
			*young_male = 0;
			*old_female = *young_female;
			*young_female = 0;
		}
	}
}
