use core::fmt;
use std::cmp::Ordering;
use rand::Rng;
use crate::graph::Graph;

/// Stores graph, ants and meta information for 
/// ACO.
///     Graph: Graph struct type contains all bag references and pheromone information
///     Ants: Collection fo Ant struct types
///     Best Path: Contains data in the order off:
///         (Tour as Vec<Bag references as usize>, cost, weight)
///     num_of_fitness_evaluations: Current number of fitness evalutations in the ACO
pub struct Colony {
    pub graph: Graph,
    pub ants: Vec<Ant>,
    pub best_path: (Vec<usize>, f64, f64),
    pub num_of_fitness_evaluations: i64,
}

impl fmt::Display for Colony {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Graph Size: {}\nNumber of Ants: {}\nBest Path Cost\\weight: {}\\{}\nBest Path: {:?}",
            self.graph.nodes,
            self.ants.len(),
            self.best_path.1,
            self.best_path.2,
            self.best_path.0.iter()
                .map(|bag| format!("{}", self.graph.graph[*bag].number))
                .collect::<Vec<String>>().join(" -> ")
        )
    }
}

impl Colony {
    /// Returns a new coloney with the given graph,
    /// best path is set to an empty vector, with 
    /// cost and weight as 0.0
    pub fn new(mut graph: Graph) -> Self {
        // Adds a uniform distribution of pheromones values to the 
        // Tau structure
        graph.initialize_tau();
        Colony { 
            graph: graph,
            ants: Vec::new(),
            best_path: (Vec::new(), 0.0, 0.0), 
            num_of_fitness_evaluations: 0,
        }
    }
    
    /// Prints the colony's data,
    /// if verbose is true then the best path is included
    pub fn print_colony(&self, verbose: bool) {
        if verbose {
            println!("Graph Size: {}\nNumber of Ants: {}\nBest Path Cost\\weight: {}\\{}\nBest Path: {:?}",
                self.graph.nodes,
                self.ants.len(),
                self.best_path.1,
                self.best_path.2,
                self.best_path.0.iter()
                    .map(|bag| format!("{}", self.graph.graph[*bag].number))
                    .collect::<Vec<String>>().join(" -> ")
            );
        } else {
            println!("Graph Size: {}\nNumber of Ants: {}\nBest Path Cost-weight: {}\\{}",
                self.graph.nodes,
                self.ants.len(),
                self.best_path.1,
                self.best_path.2.round(),
                );
        }
    }

    /// Fill the colony with new ants at random bags
    pub fn init_ants(&mut self, num_of_ants: i64) {
        self.ants = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..num_of_ants {
            let bag = rng.gen_range(0..self.graph.nodes);
            self.ants.push(Ant::birth(bag, &self.graph));
        }
    }

    /// Constructs all the ants tours. A tour is complete
    /// if no other bag can be added due to the weight 
    /// constraint
    /// Returns true when finished
    pub fn run_tours(&mut self, alpha: f64) -> bool {
        while !self.are_all_tours_finished() {
            self.time_step(alpha);
        }
        return true
    }

    /// Adds one bag to each ants tour if there is a
    /// bag within the weight constraint
    pub fn time_step(&mut self, alpha: f64) {
        for ant in self.ants.iter_mut() {
            ant.update_ant(&self.graph, alpha);
        }
    }

    /// Updates all edges through pheromone evaporation and pheromone updating
    /// evaporation_rate: Evaporation scalar
    /// p_rate: Pheromone scalar
    pub fn update_edges(&mut self, evaporation_rate: f64, p_rate: f64) {
        // Panics if edges are updates before ants have finished their tours,
        // this should never happen unless the algorithm is not running
        // as intended
        match self.set_best_tour() {
            Some(_) => {
                panic!("Ealier call to update, ants had not finished their tours!!!");
            },
            None => (),
        }
        
        // Evaporate edges
        self.graph.evaporation_edges(evaporation_rate);

        // Update pheromone levels for all edges traversed by an ant
        for ant in self.ants.iter() {
            let tour_value: f64 = ant.calculate_tour_cost(&self.graph);
            let tour_weight: f64 = ant.calcluate_tour_weight(&self.graph);
            let mut bag_i: usize = *ant.tour.get(0).unwrap();
            // Skip first bag_i
            for bag_j in ant.tour.iter().skip(1) {       
                self.graph.deposit_phero((bag_i, *bag_j), tour_value, tour_weight, p_rate);                
                bag_i = *bag_j
            }
        }
    }

    /// Finds and sets the best tour in the colony,
    /// Returns Option(None) if successful
    /// Some(String) if the tours are not finished yet
    pub fn set_best_tour(&mut self) -> Option<String>{
        if !self.are_all_tours_finished() {
            return Some("Failed: Ants have not finished their tour".to_string());
        }
        // Update the number of fitness evaluations by the number of ants, since
        // its one tour evaluation per ant tour
        self.num_of_fitness_evaluations += self.ants.len() as i64;
        // Find all the ants values
        let ants_values: Vec<f64> = self.ants.iter().map(|ant| ant.current_cost).collect();
        
        // Find the ant with the highest cost
        let top_ant: &Ant = self.ants
            .get(ants_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b)
            .unwrap_or(Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap())
            .unwrap();        
        
        // Set the colony's best tour data
        self.best_path = (
            top_ant.tour.clone(),
            top_ant.current_cost,
            top_ant.current_weight,
        );
        // Succussful return
        None
    }

    /// Checks if all ants tours are finished by checking if 
    /// any ants have any bags that they cna travell too
    /// Returns false if there are still ants with nodes left to visit
    /// if no ants cannot continue, returns true
    pub fn are_all_tours_finished(&self) -> bool {
        !self.ants.iter().any(|ant| !self.graph.get_availible_bags(
            &ant.current_bag, 
            &ant.tour, 
            ant.calculate_allowed_weight(self.graph.max_weight)
        ).is_empty())
    }

    /// Gets the average tour as the cost of 
    /// all ants tours in the colony / the number of ants
    pub fn calculate_average_cost(&self) -> f64 {
        self.calculate_total_colony_cost() / self.ants.len() as f64
    }

    /// Calculates the total cost of all ant's tours in
    /// the colony
    pub fn calculate_total_colony_cost(&self) -> f64{
        self.ants.iter().map(|ant|ant.current_cost).sum()
    }
}

/// Represents an Ant and it's meta information
/// current_bag: Index of bag in graph
/// tour: Vector of index's of bags in graph
/// current_cost: The current, cumulative cost of all bags in the tour
/// current_weight: The current, cumulative weight of all bags in the tour
pub struct Ant {
    pub current_bag: usize,
    pub tour: Vec<usize>,
    // Tour cost and weight is tracked for performance at the 
    // small cost of memory
    pub current_cost: f64,
    pub current_weight: f64,
}

impl Ant {
    /// Creates a new ant with the given bag and bag
    /// values
    pub fn birth(bag: usize, graph: &Graph) -> Self {
        Ant {
            current_bag: bag, 
            tour: vec![bag], 
            current_cost: graph.graph[bag].cost, 
            current_weight: graph.graph[bag].weight
        }
    }

    /// Update ant for time step, moving the ant from one 
    /// bag to another in teh graph
    /// Move ant from one node to the next, updating their tour
    /// working within weight constraints
    /// graph: Graph struct reference containing bags
    /// alpha: Scalar value applied to pheromone levels
    pub fn update_ant(&mut self, graph: &Graph, alpha: f64) {
        // Gets all valid bags the ant can move too
        let availible_bags: Vec<usize> = graph.get_availible_bags(
            &self.current_bag,
            &self.tour,
            self.calculate_allowed_weight(graph.max_weight)
        );        
        // If there is atleast one bag availible, add a bag to the ant's tour
        // according to the update rules in graph.select_path
        if !availible_bags.is_empty() {        
            let new_bag = graph.select_path(&self.current_bag, &availible_bags, alpha);                
            if new_bag.is_some() { 
                let new_bag = new_bag.unwrap();
                self.tour.push(new_bag);
                self.current_bag = new_bag;
                self.current_cost += graph.graph[self.current_bag].cost;
                self.current_weight += graph.graph[self.current_bag].weight;
            }
        }
    }
    
    /// Get the ant's total tour cost
    pub fn calculate_tour_cost(&self, graph: &Graph) -> f64{
        self.tour.iter().map(|bag| graph.graph[*bag].cost).sum()
    }
    
    /// Get the ant's total weight 
    pub fn calcluate_tour_weight(&self, graph: &Graph) -> f64 {
        self.tour.iter().map(|bag| graph.graph[*bag].weight).sum()
    }
    
    /// Get the allowed weight by the difference in the 
    /// max_allowed weight and the ant's current tour weight
    pub fn calculate_allowed_weight(&self, max_allowed_weight: f64) -> f64 {
        max_allowed_weight - self.current_weight
    }

    /// Prints the ant's tour in a human-readable format
    pub fn print_ants_tour(&self, graph: &Graph) {
        println!("___________________");
        for bag in self.tour.iter() {
            print!("{} -> ", graph.graph[*bag].number);
        }
        println!("Total Cost: {}", self.calculate_tour_cost(graph));
        println!("Length: {}", self.tour.len());
        println!("___________________");
    }
}


#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    /// Test the Ordering of finding the best ant out of a selection of "tour" values
    #[test]
    fn test_f64_order() {
        let ants_values = vec![0.0, 32000.32, 16.4, 100.0, 11.0];
        let top_index: usize = ants_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b)
            .unwrap_or(Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap();
        assert_eq!(top_index, 1);
    }    
}