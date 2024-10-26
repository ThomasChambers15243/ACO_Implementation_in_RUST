use std::collections::HashMap;
use std::fs::OpenOptions;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::str::FromStr;

use std::error::Error;

pub mod algorithm;
pub mod graph;
pub mod ant;

static mut CSV_INITILIZED: bool = false;

enum Parameter {
    Alpha(f64),
    Beta(f64),
    DecayRate(f64),
    PRate(f64),
    NumOfAnts(i64),
    FitnessEvals(i64),
}

impl Parameter {
    pub fn extract_parameters(parameters: HashMap<String, Parameter>) -> (f64, f64, f64, f64, i64, i64) {
        (
            parameters.get("alpha").and_then(Parameter::as_f64).unwrap(),
            parameters.get("beta").and_then(Parameter::as_f64).unwrap(),
            parameters.get("decay_rate").and_then(Parameter::as_f64).unwrap(),
            parameters.get("p_rate").and_then(Parameter::as_f64).unwrap(),
            parameters.get("num_of_ants").and_then(Parameter::as_i64).unwrap(),
            parameters.get("fitness_evals").and_then(Parameter::as_i64).unwrap(),
        )
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Parameter::Alpha(val) | Parameter::Beta(val) | Parameter::DecayRate(val) | Parameter::PRate(val) => Some(*val),
            _ => None,
        }
    }
    fn as_i64(&self) -> Option<i64> {
        match self {
            Parameter::FitnessEvals(val) | Parameter::NumOfAnts(val) => Some(*val),
            _ => None,        
        }
    }
}

fn main() {
    let choices = &["DEFAULT", "CUSTOM"];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter values or run default")
        .items(choices)
        .default(0)
        .interact()
        .unwrap();

    match choices[choice] {        
        "DEFAULT" => {
            println!("Running with DEFAULT settings...");
            // Set up defaut parameters
            let number_of_runs: i64 = 1;
            let mut parameters: HashMap<String, Parameter> = HashMap::new();
            parameters.insert(String::from("alpha"), Parameter::Alpha(1.0));
            parameters.insert(String::from("beta"), Parameter::Beta(2.5));
            parameters.insert(String::from("decay_rate"), Parameter::DecayRate(0.2));
            parameters.insert(String::from("p_rate"), Parameter::PRate(1.0));
            parameters.insert(String::from("num_of_ants"), Parameter::NumOfAnts(100));
            parameters.insert(String::from("fitness_evals"), Parameter::FitnessEvals(10000));
            let params: (f64, f64, f64, f64, i64, i64) = Parameter::extract_parameters(parameters);   
            for _ in 0..number_of_runs {                     
                let results: HashMap<String, String> = run(params);
                match write_to_csv("results.csv", params, results) {
                    Ok(_) => println!("Results written"),
                    Err(e) => println!("{}", e),
                }
            }
        }
        "CUSTOM" => {
            let parameters = get_parameters();
            let number_of_runs: i64 = input_wrapper::<i64>("Enter the number of runs for the algorithm");
            let csv_path: String = input_wrapper::<String>("Enter the CSV Path (with .csv as the suffix)");
            println!("Running with custome parameters...");
            let params: (f64, f64, f64, f64, i64, i64) = Parameter::extract_parameters(parameters);
            for _ in 0..number_of_runs {
                let results: HashMap<String, String>  = run(params);
                match write_to_csv(csv_path.as_str(), params, results) {
                    Ok(_) => println!("Results written"),
                    Err(e) => println!("{}", e),
                }
            }
        }
        _ => unreachable!("Invalid selection"),
    }
}

// Given params, runs the ACO algorithm and returns the results as a hashmap of string : string
fn run(params: (f64, f64, f64, f64, i64, i64)) -> HashMap<String, String> {
    algorithm::run(
        params.0,
        params.1,
        params.2,
        params.4,
        params.5,
        params.3,        
        true
    )
} 

// Writes ACO's results to the csv
fn write_to_csv(path: &str, params: (f64, f64, f64, f64, i64, i64), results: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    init_csv(path)?;
    
    // Open the file in append mode as to note write over previous data
    let file = OpenOptions::new().append(true).open(path)?;
    let mut wtr = csv::Writer::from_writer(file);

    let difference = results.get("initial_score").unwrap().parse::<f64>()? - results.get("final_score").unwrap().parse::<f64>()?;
    let avg_difference = results.get("initial_avg").unwrap().parse::<f64>()? - results.get("final_avg").unwrap().parse::<f64>()?;

    wtr.write_record(&[
        params.0.to_string(),
        params.1.to_string(),
        params.2.to_string(),
        params.3.to_string(),
        params.4.to_string(),
        params.5.to_string(),
        results.get("initial_score").unwrap().to_string(),
        results.get("initial_avg").unwrap().to_string(),
        results.get("final_score").unwrap().to_string(),
        results.get("final_avg").unwrap().to_string(),
        difference.trunc().to_string(),
        avg_difference.trunc().to_string(),
    ])?;
    // Flush buffer and return
    wtr.flush()?;
    Ok(())
}

// Writes the headers to the csv, wiping any previous data
fn init_csv(path: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        if !CSV_INITILIZED {
            let mut wtr = csv::Writer::from_path(path)?;
            wtr.write_record(
            &[
                "Alpha", 
                "Beta", 
                "Decay_Rate",
                "p_rate",
                "Number_Of_Ants", 
                "Fitness_Evals", 
                "Initial_fitness", 
                "Initial_avg",
                "Top_Fitness", 
                "Final_avg",
                "Best_Fitness_Difference",
                "Avg_Difference",
            ])?;
            wtr.flush()?;
            CSV_INITILIZED = true; 
        }
    }
    Ok(())
}

// Get parameters from the user through inputs
fn get_parameters() -> HashMap<String, Parameter> {
    let mut parameters_map: HashMap<String, Parameter> = HashMap::new();
    parameters_map.insert(
        "alpha".to_string(), 
        Parameter::Alpha(input_wrapper::<f64>("Enter the alpha value: "))
    );
    parameters_map.insert(
        "beta".to_string(), 
        Parameter::Beta(input_wrapper::<f64>("Enter the beta value: "))
    );
    parameters_map.insert(
        "decay_rate".to_string(), 
        Parameter::DecayRate(input_wrapper::<f64>("Enter the decay rate: "))
    );
    parameters_map.insert(
        "p_rate".to_string(),
        Parameter::PRate(input_wrapper::<f64>("Enter the pheromone rate: "))
    );
    parameters_map.insert(
        "num_of_ants".to_string(), 
        Parameter::NumOfAnts(input_wrapper::<i64>("Enter the number of ants: "))
    );
    parameters_map.insert(
        "fitness_evals".to_string(), 
        Parameter::FitnessEvals(input_wrapper::<i64>("Enter the terminal number of fitness evaluations: "))
    );
    
    parameters_map
}


// Gets user's input and parses into the correct data type
fn input_wrapper<T>(prompt: &str) -> T 
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .validate_with(|input: &String| -> Result<(),&str> {
            match input.parse::<T>() {
                Ok(_) => Ok(()),
                Err(_) => Err("Invalid input, please enter a valid number."),
            }
        })
        .interact()
        .unwrap().parse::<T>().unwrap()
}



