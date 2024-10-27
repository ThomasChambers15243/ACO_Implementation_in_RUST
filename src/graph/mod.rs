use std::convert::TryInto;
use std::path::Path;
use std::fs;
use core::fmt;
use rand::Rng;

/// Constant size of the number of bags in the text file
/// !!! Important !!!
/// Modify this carfully, depending on the BankProblem files
/// you use
const BAG_NUMBER: usize = 100;

/// Represents each bag and its meta data
/// number: Bag number
/// weight: Weight of the bag
/// cost: Value of each bag
/// Ratio: The cost/weight ratio of each bag
/// h: Pre-calculated value of each bag's ratio * beta values
///     Handled in creation of the bag
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bag {
    pub number: i64,
    pub weight: f64,
    pub cost: f64,
    pub ratio: f64,
    pub h: f64,
}

impl fmt::Display for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Weight: {}\nCost: {}", self.weight, self.cost)
    }
}

impl PartialOrd for Bag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.number.partial_cmp(&other.number)
    }
}

/// Represents the graph used to store bags and meta data.
/// Vectors are used over arrays to avoid stack overflow errors
/// with large data sets. Since vectors are only accessed, 
/// capacity change is never needed after creation so 
/// the performance loss is minimal and acceptable.
/// max_weight: The max weight constraint of the problem
/// nodes: the number of nodes in the problem
/// graph: Constant size collection of Bags with a fixed indicies
/// tau: Tau struct containing pheromone data
#[derive(Debug)]
pub struct Graph {
    pub max_weight: f64,
    pub nodes: usize,
    pub graph: Vec<Bag>,
    pub tau: Tau,
}

/// Contains the pheromones values on edges. Stores information
/// as a spares matrix. However, since Rust 2-D arrays are not
/// bi-directional, access is controlled though edge validation
/// where i < j is always true for any edge get/set operations
/// 
/// See modules tests for validation
#[derive(Debug)]
pub struct Tau {
    matrix: Vec<Vec<f64>>
}

impl Tau {
    /// Creates a new matrix to store pheromone values in
    pub fn new() -> Self {
        Tau {matrix: vec![vec![0.0; BAG_NUMBER]; BAG_NUMBER]}
    }
    
    /// Returns the raw metrix, use with caution
    pub fn get_matrix(&mut self) -> &Vec<Vec<f64>>{//[[f64; BAG_NUMBER]; BAG_NUMBER] {
        &self.matrix
    }
    
    /// Sets the value of an edge to the given f64 value
    pub fn set_edge(&mut self, bag_i: usize, bag_j: usize, value: f64) {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j] = value;
        } else {
            self.matrix[bag_j][bag_i] = value;
        }
    }
    
    /// Returns the values on a given edge
    pub fn get_edge(&self, bag_i: usize, bag_j: usize) -> f64 {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j]
        } else {
            self.matrix[bag_j][bag_i]
        }
    }

    /// Adds the given values to the given edge
    pub fn add_to_edge(&mut self, bag_i: usize, bag_j: usize, value: f64) {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j] += value;
        } else {
            self.matrix[bag_j][bag_i] += value;
        }
    }
}

impl Graph {
    /// Constructs a new graph, loading in bag problems
    /// for the given problem.
    /// Herisitc information is pre-calculated as the bags
    /// are created, for performance gains, as thisv value
    /// is constant throughout the algorithm
    /// beta: weight for herisitc bias
    pub fn construct_graph(beta: f64) -> Self {
        let (max_weight, bags) = load_data(beta);
        let nodes = bags.len();
        let graph: Vec<Bag> = bags.try_into().unwrap();        
        let tau = Tau::new();
        Graph {
            max_weight,
            nodes,
            graph,
            tau,
        }
    }

    /// Distributes a uniform pheromone values across
    /// all edges
    pub fn initialize_tau(&mut self) {
        let mut rng = rand::thread_rng();
        let bags = &self.graph;

        for i in 0..bags.len() {
            for j in 0..bags.len() {
                // Avoids pointless pheromone addition for performance gains
                if i != j {
                    self.tau.set_edge(i, j, rng.gen_range(0.1..1.0));
                }
            }
        }
    }

    /// Gets all possible bags which can be visited next,
    /// according to the given arguments
    /// current_bag: The current bag_i to be checked
    /// visited_bags: Collection of visited bags that are
    ///     unavaible for future traversal
    /// allowed_weight: The maximium weight of any future bag
    ///     according to constraints
    /// Returns empty vector if no bags are possible.
    pub fn get_availible_bags(
        &self,
        current_bag: &usize,
        visited_bags: &Vec<usize>,
        allowed_weight: f64,
    ) -> Vec<usize> {
        self.graph
            .iter().enumerate()
            .filter(|&bag| {
                bag.0 != *current_bag
                && !visited_bags.contains(&&bag.0)
                && bag.1.weight <= allowed_weight
            })
            .map(|bag| bag.0)
            .collect()
    }

    /// Uses fitness proportional selection (roulette wheel) to
    /// select the next bag, given
    /// bag_i: The current bag
    /// availible_bags: All bags that can be visited next
    /// alpha: Scalar weight for edge's pheromones
    /// Returns Some(index to bag in graph)
    /// 
    /// See modules tests for validation
    pub fn select_path(
        &self,
        bag_i: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> Option<usize> {
        // If there is only one bag left, then just
        // return that one for faster performance
        if availible_bags.len() == 1 {
            Some(availible_bags[0])
        } else {
            // Gets the wheel with calculated, ranked probabilities
            let wheel: Vec<f64> = self.create_selection_wheel(bag_i, availible_bags, alpha);
            // Gets a random choice. Range is upto 1 since all ranks sum up to 1
            let choice: f64 = rand::thread_rng().gen_range(0.0..=1.0);
            // Returns the correct bag given the wheel and random choice
            availible_bags
                .iter()
                .zip(wheel.iter())
                .find(|(_, &rank)| choice <= rank)
                .map(|(bag, _)| *bag)
        }
    }

    /// Creates a routllet wheel given
    /// bag_i: The current bag
    /// availible_bags: All bags that can be visited next
    /// alpha: Scalar weight for edge's pheromones
    /// Returns a vector of f64 probabilities
    fn create_selection_wheel(
        &self,
        bag_i: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> Vec<f64> {        
        // Collect probabilities
        let probabilities: Vec<f64> = availible_bags
            .iter()
            .map(|bag| self.calculate_edge_probability(bag_i, bag, availible_bags, alpha))
            .collect();
                
        // Collect cumulative probabbilities
        probabilities
            .iter()
            .scan(0.0, |cum_sum, &p| {
                *cum_sum += p;
                Some(*cum_sum)
            })
            .collect()
    }

    /// Calculates the porbability of each edge, 
    /// according to the selection rules, given
    /// bag_i: The current bag index
    /// bag_j: The next bag index
    /// availible_bags: All possible bags to be visited
    /// alpha: Scalar weight for edge's pheromones
    /// Returns a f64 probability
    fn calculate_edge_probability(
        &self,
        bag_i: &usize,
        bag_j: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> f64 {
        // Update Rule
        // H with Beta is precomputed for performance gains
        // so h is the ratio of cost/weight
        // 
        // P_ij for ant K =
        // 
        // (tau_ji^alpha * h_ij^beta)
        // --------------------------------
        // Sum_J_i^k[ (tau_j^alpha * h_j^beta) ]
        // 
        // otherwise
        // 0
        let t: f64 = self.tau.get_edge(*bag_i, *bag_j).powf(alpha);
        let h: f64 = self.graph[*bag_j].h;
        
        let sum_of_availible_bags: f64 = availible_bags
            .iter()
            .map(|bag| {
                let t = self.tau.get_edge(*bag_i, *bag).powf(alpha);
                t * self.graph[*bag].h
            })
            .sum::<f64>();
        // Compute the edge probability
        (t * h) / sum_of_availible_bags
    }

    /// Evaporate pheromones from edges according to 
    /// the ecaporation_rate. This ACO implemenation uses
    /// the given rate AS the direct scalar rate, rather than
    /// (1-P).
    pub fn evaporation_edges(&mut self, evaporation_rate: f64) {
        for i in 0..100 {
            for j in 0..100 {
                let value = self.tau.get_edge(i, j);
                self.tau.set_edge(i, j, value * evaporation_rate);
            }
        }
    }

    /// Deposits pheromones additions on edges
    /// Heristic is based upon the ratio of cost-to-weight, 
    /// also used by KRZYSZTOF SCHIFF as 
    /// AKA2 https://repozytorium.biblos.pk.edu.pl/redo/resources/30706/file/suwFiles/SchiffK_AntColony.pdf
    /// Inuition:
    /// While minimising weight is not an objective, bags with a lower weight allow for more bags,
    /// which allows for a higher potential of a higher cost. However, just minimising weight would
    /// ignore the actual cost objective. Therefore, the ratio of cost-weight is taken, since
    /// the a higher ratio would suggust a higher value bag, in respect to constructing a better tour.
    /// The pheromone value is incremented by the tour's total cost divided by the tour's total weight.
    /// The tour's cost is multiplied by the pheromone weight, allowing for modification through
    /// experimeants without affecting the heristic's format.
    pub fn deposit_phero(&mut self, edge: (usize, usize), tour_value: f64, tour_weight: f64, p_rate: f64) {
        let value = (tour_value*p_rate) / tour_weight;
        self.tau.add_to_edge(edge.0, edge.1, value);
    }
}

/// Loads data from the given text files.
/// !!! IMPORTANT !!!
/// 1. To run, ensure the path is the correct path to the problem's
///    .txt file, otherwise the file cannot be read in and
/// 2. Ensure the problem .txt file is in the exact format is was given
///    in the problem set.
fn load_data(beta: f64) -> (f64, Vec<Bag>) {
    let path = Path::new("src\\BankProblem.txt");
    //let path = Path::new("/home/tomchambers/Documents/Exeter/409_aco/src/BankProblem.txt");
    println!("{:?}", path.to_str());
    let data = fs::read_to_string(path).expect("Unable to read file");

    let mut split_data: Vec<String> = data
        .split('\n')
        .map(|line| line.strip_suffix("\r").unwrap_or(line).trim().to_string())
        .collect();

    let mut bags: Vec<Bag> = Vec::new();
    let mut data_itre = split_data.iter();

    let mut number: i64 = 0;
    while let Some(data_value) = data_itre.next() {
        if data_value.contains("bag") {
            let weight = data_itre
                .next()
                .unwrap()
                .strip_prefix("weight: ")
                .unwrap()
                .parse()
                .unwrap();
            let cost = data_itre
                .next()
                .unwrap()
                .strip_prefix("value: ")
                .unwrap()
                .parse()
                .unwrap();
            let ratio = cost / weight;
            bags.push(Bag {
                number,
                weight,
                cost,
                ratio,
                h: ratio.powf(beta),
            });
            number += 1;
        }
    }
    (
        split_data.remove(0).strip_prefix("security van capacity: ").unwrap().parse().unwrap(),
        bags,
    )
}

/// Mutli tests to ensure key functions within ACO work as intended.
#[cfg(test)]
mod test  {
    use std::vec;

    /// Tests the tau's edge mangement system handles edges correctly
    use super::*;
    #[test]
    fn tau() {
        let mut tau = Tau::new();
        tau.set_edge(10, 15, 100.0);
        assert_eq!(tau.get_edge(10, 15), 100.0);
        assert_eq!(tau.get_edge(10, 15), tau.get_edge(15, 10));
        tau.add_to_edge(15, 10, 100.0);
        assert_eq!(tau.get_edge(10, 15), 200.0);
    }

    /// Tests that the selection wheel correctly constructs and selects bags
    /// based on ranked probability selection.
    #[test]
    fn path_selection() {
        let probabilities: Vec<f64> = vec![0.2, 0.3, 0.1, 0.4];
        let availible_bags: Vec<i32> = vec![1,2,3,4];
        //probabilities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let wheel: Vec<f64> = probabilities
        .iter()
        .scan(0.0, |cum_sum, &p| {
            *cum_sum += p;
            Some(*cum_sum)
        })
        .collect();
    
        println!("{:?}", probabilities);
        println!("{:?}", wheel);
        // [0.2, 0.3, 0.1, 0.4]
        // [0.2, 0.5, 0.6, 1.0]

        let choice = 0.1;
        assert_eq!(availible_bags
            .iter()
            .zip(wheel.iter())
            .find(|(_, &rank)| choice <= rank)
            .map(|(bag, _)| *bag).unwrap(), 
            1);
        let choice = 0.3;
        assert_eq!(availible_bags
            .iter()
            .zip(wheel.iter())
            .find(|(_, &rank)| choice <= rank)
            .map(|(bag, _)| *bag).unwrap(), 
            2);
        let choice = 0.55;
        assert_eq!(availible_bags
            .iter()
            .zip(wheel.iter())
            .find(|(_, &rank)| choice <= rank)
            .map(|(bag, _)| *bag).unwrap(), 
            3);
        let choice = 0.7;
        assert_eq!(availible_bags
            .iter()
            .zip(wheel.iter())
            .find(|(_, &rank)| choice <= rank)
            .map(|(bag, _)| *bag).unwrap(), 
            4);            
    }
}