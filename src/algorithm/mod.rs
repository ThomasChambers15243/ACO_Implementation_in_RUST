use std::collections::HashMap;
// ACO mods
use crate::graph::Graph;
use crate::ant::Colony;
// Progress Bar
use indicatif::ProgressBar;

/// Runs the ACO algorithm with given parameters
///     alpha: Weight for edge bias
///     beta: Weight for heristic bias
///     evaporation_rate: Direct value applied to all edges, NOT (1 - evaporation_rate)
///         e.g. edge_phero * evaporation_rate
///     num_of_ants: The number of ants to be used
///     Fitness_evals: The number of fitness evalutations as a terminal condition
///     p_rate: Scalar applied to the pheromones applied to each edge
///     verbose: True if extra infomation should be printed about the algorithm
pub fn run(
        alpha: f64, 
        beta: f64,
        evaporation_rate: f64, 
        num_of_ants:i64, 
        fitness_evals: i64, 
        p_rate: f64, 
        verbose: bool
    ) -> HashMap<String, String> {
    // Stores the results of the ACO
    let mut results:  HashMap<String, String> = HashMap::new();
    
    // Init the colony, 
    let mut colony: Colony = init_aco(num_of_ants, beta);
    
    // Progress bar is set to the terminal condition
    let bar = ProgressBar::new(fitness_evals as u64);
    
    // Run one search based on random phero values
    colony.run_tours(alpha);
    colony.update_edges(evaporation_rate, p_rate);

    // Add initial search for comparison with final search
    results.insert("initial_score".to_string(), colony.best_path.1.to_string());
    results.insert("initial_avg".to_string(), colony.calculate_average_cost().to_string());
    if verbose { write_verbose(&colony)}

    // Run the ACO until the number of evaluations has been met
    while colony.num_of_fitness_evaluations < fitness_evals {
        colony.init_ants(num_of_ants);
        colony.run_tours(alpha);
        colony.update_edges(evaporation_rate, p_rate);
        if verbose { bar.set_position(colony.num_of_fitness_evaluations as u64); }
    }
    if verbose { write_verbose(&colony)}

    // Update results with final scores
    results.insert("final_score".to_string(), colony.best_path.1.to_string());
    results.insert("final_avg".to_string(), colony.calculate_average_cost().to_string());
    // Return Results
    results
}


/// Creates the graph and colony for the ACO to
/// perform with
fn init_aco(num_of_ants:i64, beta: f64) -> Colony{
    let graph: Graph = Graph::construct_graph(beta);
    let mut colony = Colony::new(graph);
    colony.init_ants(num_of_ants);
    colony
}

/// Write the conely and average cost
fn write_verbose(colony: &Colony) {
    colony.print_colony(false);
    println!("Average Cost: {}", colony.calculate_average_cost());  
}