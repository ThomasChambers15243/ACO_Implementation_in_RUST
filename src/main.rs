use std::collections::HashMap;
use std::fs::OpenOptions;
use std::str::FromStr;
use std::error::Error;
// Handles CLI inputs
use dialoguer::{theme::ColorfulTheme, Input, Select};
// Delcares mods for use in modules
pub mod algorithm;
pub mod graph;
pub mod ant;
pub mod research_set;
use research_set::ResearchSet;

/// Static to track csv creation as to not overwrite the csv headers
/// !!! Important !!!
/// If the csv file has data written which should not be overwritten
/// set this too true, then all data will be appended and the headers
/// will not be changed and re-written
static mut CSV_INITILIZED: bool = true;

/// Handles all parameter inputs and types of f64 | i64
#[derive(Clone)]
pub enum Parameter {
    Alpha(f64),
    Beta(f64),
    EvaporationRate(f64),
    PRate(f64),
    NumOfAnts(i64),
    FitnessEvals(i64),
}

impl Parameter {
    /// Given a hashmap of parameters, extracts the params into the correctly formatted 
    /// collection of data types, in the order of 
    /// (
    ///  f64: alpha,
    ///  f64: beta,
    ///  f64: evaporation_rate,
    ///  f64: pheromone_rate,
    ///  i64: num_of_ants,
    ///  i64: fitness_evals
    /// )
    pub fn extract_parameters(parameters: &HashMap<String, Parameter>) -> (f64, f64, f64, f64, i64, i64) {
        (
            parameters.get("alpha").and_then(Parameter::as_f64).unwrap(),
            parameters.get("beta").and_then(Parameter::as_f64).unwrap(),
            parameters.get("evaporation_rate").and_then(Parameter::as_f64).unwrap(),
            parameters.get("p_rate").and_then(Parameter::as_f64).unwrap(),
            parameters.get("num_of_ants").and_then(Parameter::as_i64).unwrap(),
            parameters.get("fitness_evals").and_then(Parameter::as_i64).unwrap(),
        )
    }
    /// Extracts the f64 from the parameter
    fn as_f64(&self) -> Option<f64> {
        match self {
            Parameter::Alpha(val) | Parameter::Beta(val) | Parameter::EvaporationRate(val) | Parameter::PRate(val) => Some(*val),
            _ => None,
        }
    }
    /// Extracts the i64 from the parameter
    fn as_i64(&self) -> Option<i64> {
        match self {
            Parameter::FitnessEvals(val) | Parameter::NumOfAnts(val) => Some(*val),
            _ => None,        
        }
    }
}

fn main() {
    // Constant choices for algorithm running
    let choices = &["DEFAULT", "CUSTOM", "EXPERIMENT"];

    let choice = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter values or run default")
        .items(choices)
        .default(0)
        .interact()
        .unwrap();

    match choices[choice] {        
        // Edit these to change default parameters
        "DEFAULT" => {
            let mut parameters: HashMap<String, Parameter> = HashMap::new();
            parameters.insert(String::from("alpha"), Parameter::Alpha(1.0));
            parameters.insert(String::from("beta"), Parameter::Beta(2.0));
            parameters.insert(String::from("evaporation_rate"), Parameter::EvaporationRate(0.1));
            parameters.insert(String::from("p_rate"), Parameter::PRate(1.0));
            parameters.insert(String::from("num_of_ants"), Parameter::NumOfAnts(20));
            parameters.insert(String::from("fitness_evals"), Parameter::FitnessEvals(100));
            let number_of_runs: i64 = 1;
            let path: &str = "csv/results.csv";
            // Runs algorithm with default params
            println!("Running with DEFAULT settings...");
            run_experiment(&parameters, path, number_of_runs, 1);
        },
        "EXPERIMENT" => {
            let number_of_runs: i64 = 5;
            let mut path = "csv/results_ant_num.csv";            
            
            let experiment_params: Vec<HashMap<String, Parameter>> = ResearchSet::set_ant_number_params(vec![2,5,10,15,20,30,50,100]);
            for (parameter_run, parameters) in experiment_params.into_iter().enumerate() {
                run_experiment(&parameters, path, number_of_runs, parameter_run+1);
            }
            
            path = "csv/results_evaporation.csv";
            let experiment_params: Vec<HashMap<String, Parameter>> = ResearchSet::set_evaporation_params(vec![0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8]);
            for (parameter_run, parameters) in experiment_params.into_iter().enumerate() {
                run_experiment(&parameters, path, number_of_runs, parameter_run+1);
            }

            path = "csv/results_p_rate.csv";
            let experiment_params: Vec<HashMap<String, Parameter>> = ResearchSet::set_p_rate_params(vec![0.5,1.0,2.0,3.0,4.0,5.0,6.0,7.0]);
            for (parameter_run, parameters) in experiment_params.into_iter().enumerate() {
                run_experiment(&parameters, path, number_of_runs, parameter_run+1);
            }
            
        },
        "CUSTOM" => {
            // User enters custom params with validation for data types
            let parameters = get_parameters();
            let number_of_runs: i64 = input_wrapper::<i64>("Enter the number of runs for the algorithm");
            let path: String = input_wrapper::<String>("Enter the CSV Path (with .csv as the suffix)");
            println!("Running with custome parameters...");
            // Runs algorithm with default params
            run_experiment(&parameters, path.as_str(), number_of_runs, 1);
        }
        _ => unreachable!("Invalid selection"),
    }
}

fn run_experiment(parameters: &HashMap<String, Parameter>, path:&str, number_of_runs: i64, parameter_run: usize) {
    for _ in 0..number_of_runs {
        let params: (f64, f64, f64, f64, i64, i64) = Parameter::extract_parameters(parameters);
        let results: HashMap<String, String> = run(params);
        match write_to_csv(path, params, results, parameter_run) {
            Ok(_) => println!("Results written"),
            Err(e) => println!("{}", e),
        }
    }
}

/// Given params, runs the ACO algorithm and returns the results as a hashmap of string : string
/// params in the order of 
/// (
///  f64: alpha,
///  f64: beta,
///  f64: evaporation_rate,
///  f64: pheromone_rate,
///  i64: num_of_ants,
///  i64: fitness_evals
/// )
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
fn write_to_csv(path: &str, params: (f64, f64, f64, f64, i64, i64), results: HashMap<String, String>, parameter_run: usize) -> Result<(), Box<dyn Error>> {
    init_csv(path)?;
    
    // Open the file in append mode as to note write over previous data
    let file = OpenOptions::new().append(true).open(path)?;
    let mut wtr = csv::Writer::from_writer(file);

    let difference = results.get("final_score").unwrap().parse::<f64>()? - results.get("initial_score").unwrap().parse::<f64>()?;
    let avg_difference = results.get("final_avg").unwrap().parse::<f64>()? - results.get("initial_avg").unwrap().parse::<f64>()?;
    
    // Write record
    wtr.write_record(&[
        parameter_run.to_string(),
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

/// Writes the headers to the csv, wiping any previous data
fn init_csv(path: &str) -> Result<(), Box<dyn Error>> {
    // Writes the headers to the csv files
    // Unsafe due to the modification of a static, mutable variables - CSV_INITILIZED
    unsafe {
        if !CSV_INITILIZED {
            let mut wtr = csv::Writer::from_path(path)?;
            wtr.write_record(
            &[
                "Parameter",
                "Alpha", 
                "Beta", 
                "Evaporation_Rate",
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

/// Get parameters from the user through inputs
/// Validates all inputs to ensure correct data types
/// Returns hashmap of paramater name to Parameter enum
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
        "evaporation_rate".to_string(), 
        Parameter::EvaporationRate(input_wrapper::<f64>("Enter the evaporation rate: "))
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


/// Gets user's input and parses into the correct data type
/// Takes in the input for the user as a &str
fn input_wrapper<T>(prompt: &str) -> T 
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        // Validates against the generic data type 
        .validate_with(|input: &String| -> Result<(),&str> {
            match input.parse::<T>() {
                Ok(_) => Ok(()),
                Err(_) => Err("Invalid input, please enter a valid number."),
            }
        })
        .interact()
        .unwrap().parse::<T>().unwrap()
}
