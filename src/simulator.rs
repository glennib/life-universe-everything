use std::collections::BTreeMap;
use std::collections::HashMap;

macro_rules! newtype {
	($name:ident($inner:ty)) => {
		#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
		pub struct $name(pub $inner);

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				std::fmt::Display::fmt(&self.0, f)
			}
		}
	};
}

newtype!(Year(i32));
newtype!(Age(u8));

#[derive(Debug, Clone)]
pub struct AgeGenderMap(pub HashMap<(Age, Gender), Count>);
#[derive(Debug, Clone, Default)]
pub struct CohortFertility(pub BTreeMap<Year, CohortData>);

impl CohortFertility {
	pub fn avg(&self) -> f64 {
		let sum: f64 = self.0.values().map(|cd| cd.ratio()).sum();
		let count = self.0.len() as f64;
		sum / count
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CohortData {
	pub females: Count,
	pub births: Count,
}

impl CohortData {
	pub fn ratio(self) -> f64 {
		self.births as f64 / self.females as f64
	}
}

impl AgeGenderMap {
	pub fn count(&self) -> Count {
		self.0.values().sum()
	}

	#[allow(unused)]
	pub fn count_gender(&self, gender: Gender) -> Count {
		self.0
			.iter()
			.filter_map(|(&(_, g), &count)| (g == gender).then_some(count))
			.sum()
	}

	#[allow(unused)]
	pub fn count_age(&self, age: Age) -> Count {
		self.0
			.iter()
			.filter_map(|(&(a, _), &count)| (a == age).then_some(count))
			.sum()
	}

	#[allow(unused)]
	pub fn count_age_gender(&self, age: Age, gender: Gender) -> Count {
		self.0[&(age, gender)]
	}
}

pub type Count = u64;

#[derive(Ord, PartialOrd, PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Gender {
	Male,
	Female,
}

pub struct SimulationResult {
	pub initial_population: AgeGenderMap,
	pub final_population: AgeGenderMap,
	pub cohort_fertility: CohortFertility,
	pub timeline: BTreeMap<Year, TimelineData>,
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineData {
	pub males: Count,
	pub females: Count,
}

impl TimelineData {
	pub fn sum(&self) -> Count {
		self.males + self.females
	}
}

#[derive(Clone, Copy, Debug)]
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
		let mut timeline = BTreeMap::new();
		timeline.insert(
			initial_year,
			TimelineData {
				males: population.map.count_gender(Gender::Male),
				females: population.map.count_gender(Gender::Female),
			},
		);

		for year in 0..self.n_years {
			let year = Year(initial_year.0 + i32::from(year));
			population.propagate_age();
			population.handle_births(year);
			population.handle_deaths();
			timeline.insert(
				Year(year.0 + 1),
				TimelineData {
					males: population.map.count_gender(Gender::Male),
					females: population.map.count_gender(Gender::Female),
				},
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
		let map = (0..=(parameters.max_age.0 + 1))
			.flat_map(|age| {
				let age = Age(age);
				let rel_freq = age_relative_frequency(age, parameters.max_age);
				let count_each =
					((rel_freq * (parameters.initial_population as f64)) * 0.5) as Count;
				[
					((age, Gender::Male), count_each),
					((age, Gender::Female), count_each),
				]
			})
			.collect();
		let map = AgeGenderMap(map);
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
			.0
			.iter()
			.filter(|&(&(_age, gender), _females)| gender == Gender::Female)
			.map(|(&(age, _gender), &females)| {
				(
					age,
					(birth_probability_one_year(age.0, self.parameters.target_total_fertility_rate)
						* (females as f64)) as Count,
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
		*self.map.0.get_mut(&(Age(0), Gender::Female)).unwrap() += females;
		*self.map.0.get_mut(&(Age(0), Gender::Male)).unwrap() += males;
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

		for ((age, gender), count) in self.map.0.iter_mut() {
			let probability = death_probability_one_year(
				*age,
				*gender,
				self.parameters.max_age,
				self.parameters.infant_mortality_rate,
			);
			let deaths = (*count as f64 * probability).round();
			*count -= deaths as Count;
		}
	}

	fn propagate_age(&mut self) {
		for age in (0..=self.parameters.max_age.0).rev() {
			let [old_male, old_female, young_male, young_female] = self
				.map
				.0
				.get_disjoint_mut([
					&(Age(age + 1), Gender::Male),
					&(Age(age + 1), Gender::Female),
					&(Age(age), Gender::Male),
					&(Age(age), Gender::Female),
				])
				.map(Option::unwrap);
			*old_male = *young_male;
			*young_male = 0;
			*old_female = *young_female;
			*young_female = 0;
		}
	}
}
