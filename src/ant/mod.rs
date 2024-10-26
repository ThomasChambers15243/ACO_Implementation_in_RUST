use crate::graph::{self, Bag, Graph};
use core::fmt;
use std::cmp::Ordering;
use rand::Rng;


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
    pub fn new(mut graph: Graph) -> Self {
        graph.initialize_tau();
        Colony { 
            graph: graph,
            ants: Vec::new(),
            // Best past tour, Cost/Weight
            best_path: (Vec::new(), 0.0, 0.0), 
            num_of_fitness_evaluations: 0,
        }
    }

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

    // Fill the ants with actual ants at random bags
    pub fn init_ants(&mut self, num_of_ants: i64) {
        self.ants = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..num_of_ants {
            let bag = rng.gen_range(0..self.graph.nodes);
            self.ants.push(Ant::birth(bag, &self.graph));
        }
        
    }

    // TODO: Possible optimization here
    // Dont run through ants twice
    pub fn run_tours(&mut self, alpha: f64) -> bool {
        while !self.are_all_tours_finished() {
            self.time_step(alpha);
        }
        return true
    }

    // Runs all ants over their tour
    // Returns 
    //  True if all ants have reached terminal state
    //  False if there are still ants exploring the graph
    pub fn time_step(&mut self, alpha: f64) {
        for ant in self.ants.iter_mut() {
            ant.update_ant(&self.graph, alpha);
        }
    }


    pub fn update_edges(&mut self, decay_rate: f64, p_rate: f64) {
        match self.set_best_tour() {
            Some(_) => {
                panic!("Ealier call to update, ants had not finished their tours!!!");
            },
            None => (),
        }
        
        // Update phero levels for all edges traversed by an ant
        for ant in self.ants.iter() {
            let tour_value: f64 = ant.calculate_tour_cost(&self.graph);
            let tour_weight: f64 = ant.calcluate_tour_weight(&self.graph);
            let mut bag_i: usize = *ant.tour.get(0).unwrap();
            // Skip first bag_i
            for bag_j in ant.tour.iter().skip(1) {
                self.graph.deposit_phero((bag_i, *bag_j), tour_value, tour_weight, p_rate, decay_rate);
                bag_i = *bag_j
            }
        }
    }

    // Returns true if successful
    // Returns false if tours are not finished
    pub fn set_best_tour(&mut self) -> Option<String>{
        if !self.are_all_tours_finished() {
            return Some("Failed: Ants have not finished their tour".to_string());
        }
        self.num_of_fitness_evaluations += self.ants.len() as i64;
        let ants_values: Vec<f64> = self.ants.iter().map(|ant| ant.current_cost).collect();

        let top_index: usize = ants_values
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b)
            .unwrap_or(Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap();

        let best_path: Vec<usize> = self.ants
            .get(top_index).unwrap()
            .tour.iter()
            .map(|bag| *bag)
            .collect();

        let path_weight = Colony::calcluate_tour_weight(&best_path, &self.graph);
        self.best_path = (
            best_path,
            *ants_values.get(top_index).unwrap(),
            path_weight,
        );
        None
    }

    // Returns true if there are still ants with nodes left to visit
    // if no ants cannot continue, returns true
    pub fn are_all_tours_finished(&self) -> bool {
        !self.ants.iter().any(|ant| !self.graph.get_availible_bags(
            &ant.current_bag, 
            &ant.tour, 
            ant.calculate_allowed_weight(self.graph.max_weight)
        ).is_empty())
    }

    pub fn calculate_average_cost(&self) -> f64 {
        self.calculate_total_colony_cost() / self.ants.len() as f64
    }

    pub fn calculate_total_colony_cost(&self) -> f64{
        self.ants.iter().map(|ant|ant.current_cost).sum()
    }

    fn calcluate_tour_weight(tour: &Vec<usize>, graph: &Graph) -> f64{
        tour.iter().map(|bag| graph.graph[*bag].weight).sum()
    }
}

pub struct Ant {
    pub current_bag: usize,
    pub tour: Vec<usize>,
    // Tour cost and weight is tracked for performance at the 
    // small cost of memory
    pub current_cost: f64,
    pub current_weight: f64,
}

impl Ant {
    pub fn birth(bag: usize, graph: &Graph) -> Self {
        Ant {
            current_bag: bag, 
            tour: vec![bag], 
            current_cost: graph.graph[bag].cost, 
            current_weight: graph.graph[bag].weight
        }
    }

    // Update ant for time step
    // Move ant from one node to the next, updating their tour
    // working within constraints
    pub fn update_ant(&mut self, graph: &Graph, alpha: f64) {
        let availible_bags: Vec<usize> = graph.get_availible_bags(
            &self.current_bag,
            &self.tour,
            self.calculate_allowed_weight(graph.max_weight)
        );        
        let new_bag = graph.select_path(&self.current_bag, &availible_bags, alpha);                
        if new_bag.is_some() { 
            let new_bag = new_bag.unwrap();
            self.tour.push(new_bag);
            self.current_bag = new_bag;
            self.current_cost += graph.graph[self.current_bag].cost;
            self.current_weight += graph.graph[self.current_bag].weight;
        }
    }
    
    pub fn calculate_tour_cost(&self, graph: &Graph) -> f64{
        self.tour.iter().map(|bag| graph.graph[*bag].cost).sum()
    }
    
    pub fn calcluate_tour_weight(&self, graph: &Graph) -> f64 {
        self.tour.iter().map(|bag| graph.graph[*bag].weight).sum()
    }
    
    pub fn calculate_allowed_weight(&self, max_allowed_weight: f64) -> f64 {
        max_allowed_weight - self.current_weight
    }

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