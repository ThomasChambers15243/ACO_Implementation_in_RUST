use std::collections::HashMap;

use crate::graph::Graph;
use crate::ant::Colony;
use indicatif::ProgressBar;

pub fn run(alpha: f64, beta: f64, decay_rate: f64, num_of_ants:i64, fitness_evals: i64, p_rate: f64, verbose: bool) -> HashMap<String, String> {
    // Stores the results of the ACO
    let mut results:  HashMap<String, String> = HashMap::new();
    // Init Progress bar and colony, 
    // progress bar is set to the terminal condition
    let mut colony: Colony = init_aco(num_of_ants, beta);
    let bar = ProgressBar::new(fitness_evals as u64);
    // Run one search based on random phero values
    colony.run_tours(alpha);
    colony.update_edges(decay_rate, p_rate);
    // Add initial search for comparison with final search
    results.insert("initial_score".to_string(), colony.best_path.1.to_string());
    results.insert("initial_avg".to_string(), colony.calculate_average_cost().to_string());
    // Print results to console
    if verbose { write_verbose(&colony)}
    // Run the ACO until the number of evaluations has been met
    while colony.num_of_fitness_evaluations < fitness_evals {
        colony.init_ants(num_of_ants);
        colony.run_tours(alpha);
        colony.update_edges(decay_rate, p_rate);
        bar.set_position(colony.num_of_fitness_evaluations as u64);
        println!("In While");
    }
    // Print results to console
    if verbose { write_verbose(&colony)}
    // Update results with final scores
    results.insert("final_score".to_string(), colony.best_path.1.to_string());
    results.insert("final_avg".to_string(), colony.calculate_average_cost().to_string());
    results
}



fn init_aco(num_of_ants:i64, beta: f64) -> Colony{
    let graph: Graph = Graph::construct_graph(beta);
    let mut colony = Colony::new(graph);
    colony.init_ants(num_of_ants);
    colony
}

fn write_verbose(colony: &Colony) {
    colony.print_colony(false);
    println!("Average Cost: {}", colony.calculate_average_cost());  
}