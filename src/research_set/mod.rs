use std::collections::HashMap;

use crate::Parameter;

/// Sets up parameter's for the experiements
/// 
/// If the experiment option is chosen in the menu, then
/// the parameters for the tests will be pulled from here
/// Edit values from here for the experiement
pub struct ResearchSet {}

impl ResearchSet {
    /// Sets the Params for the nunber of ants
    pub fn set_ant_number_params(values: Vec<i64>) -> Vec<HashMap<String, Parameter>> {

        let mut default: HashMap<String, Parameter> = ResearchSet::set_default_parameters();

        let mut experiment: Vec<HashMap<String, Parameter>> = Vec::new();

        for value in values {
            if let Some(rate) = default.get_mut("num_of_ants") {
                *rate = Parameter::NumOfAnts(value);
            }        
            experiment.push(
                default.clone()
            );
        }
        experiment
    }

    /// Sets the Params for the evaporation rate
    pub fn set_evaporation_params(values: Vec<f64>) -> Vec<HashMap<String, Parameter>> {

        let mut default: HashMap<String, Parameter> = ResearchSet::set_default_parameters();

        let mut experiment: Vec<HashMap<String, Parameter>> = Vec::new();

        for value in values {
            if let Some(rate) = default.get_mut("evaporation_rate") {
                *rate = Parameter::EvaporationRate(value);
            }        
            experiment.push(
                default.clone()
            );
        }
        experiment
    }

    /// Sets the Params for pheromone rate
    pub fn set_p_rate_params(values: Vec<f64>) -> Vec<HashMap<String, Parameter>> {

        let mut default: HashMap<String, Parameter> = ResearchSet::set_default_parameters();

        let mut experiment: Vec<HashMap<String, Parameter>> = Vec::new();

        for value in values {
            if let Some(rate) = default.get_mut("p_rate") {
                *rate = Parameter::PRate(value);
            }        
            experiment.push(
                default.clone()
            );
        }
        experiment
    }

    /// Sets the default parameters to be used in conjunction with
    /// the dependent parameter being tested
    pub fn set_default_parameters() -> HashMap<String, Parameter> {
        let mut parameters: HashMap<String, Parameter> = HashMap::new();
        parameters.insert(String::from("alpha"), Parameter::Alpha(1.0));
        parameters.insert(String::from("beta"), Parameter::Beta(2.0));
        parameters.insert(String::from("evaporation_rate"), Parameter::EvaporationRate(0.1));
        parameters.insert(String::from("p_rate"), Parameter::PRate(1.0));
        parameters.insert(String::from("num_of_ants"), Parameter::NumOfAnts(50));
        parameters.insert(String::from("fitness_evals"), Parameter::FitnessEvals(10000));
        
        parameters
    }

}