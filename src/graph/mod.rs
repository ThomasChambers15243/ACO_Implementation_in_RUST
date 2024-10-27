use core::fmt;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use rand::Rng;
use std::convert::TryInto;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bag {
    pub number: i64,
    pub weight: f64,
    pub cost: f64,
    pub ratio: f64,
    pub h: f64,
}

impl Eq for Bag {}

impl Hash for Bag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
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

#[derive(Debug)]
pub struct Graph {
    pub max_weight: f64,
    pub nodes: usize,
    pub graph: [Bag; 150], 
    pub tau: Tau,
    pub tracking: Vec<f64>
}

#[derive(Debug)]
pub struct Tau {
    matrix: [[f64;150];150]
}

impl Tau {
    pub fn new() -> Self {
        Tau {matrix: [[0.0; 150]; 150]}
    }
    pub fn get_matrix(&mut self) -> [[f64;150];150] {
        self.matrix
    }
    pub fn set_edge(&mut self, bag_i: usize, bag_j: usize, value: f64) {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j] = value;
        } else {
            self.matrix[bag_j][bag_i] = value;
        }
    }
    pub fn get_edge(&self, bag_i: usize, bag_j: usize) -> f64 {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j]
        } else {
            self.matrix[bag_j][bag_i]
        }
    }
    pub fn add_to_edge(&mut self, bag_i: usize, bag_j: usize, value: f64) {
        if bag_i < bag_j {
            self.matrix[bag_i][bag_j] += value;
        } else {
            self.matrix[bag_j][bag_i] += value;
        }
    }
}

impl Graph {
    pub fn construct_graph(beta: f64) -> Self {
        let (max_weight, bags) = load_data(beta);
        let nodes = bags.len();
        let graph: [Bag; 150] = bags.try_into().unwrap();        
        let tau = Tau::new();
        Graph {
            max_weight,
            nodes,
            graph,
            tau,
            tracking: Vec::new(),
        }
    }

    pub fn initialize_tau(&mut self) {
        let mut rng = rand::thread_rng();
        let bags = &self.graph;

        for i in 0..bags.len() {
            for j in 0..bags.len() {
                if i != j {
                    self.tau.set_edge(i, j, rng.gen_range(0.1..1.0));
                }
            }
        }
    }

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

    pub fn select_path(
        &self,
        bag_i: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> Option<usize> {
        if availible_bags.len() == 1 {
            Some(availible_bags[0])
        } else {
            let wheel: Vec<f64> = self.create_selection_wheel(bag_i, availible_bags, alpha);
            let choice: f64 = rand::thread_rng().gen_range(0.0..=1.0);
            availible_bags
                .iter()
                .zip(wheel.iter())
                .find(|(_, &rank)| choice <= rank)
                .map(|(bag, _)| *bag)
        }
    }

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

    fn calculate_edge_probability(
        &self,
        bag_i: &usize,
        bag_j: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> f64 {
        let t: f64 = self.tau.get_edge(*bag_i, *bag_j).powf(alpha);
        let h: f64 = self.graph[*bag_j].h;
        
        let sum_of_availible_bags: f64 = availible_bags
            .iter()
            .map(|bag| {
                let t = self.tau.get_edge(*bag_i, *bag).powf(alpha);
                t * self.graph[*bag].h
            })
            .sum::<f64>();
        (t * h) / sum_of_availible_bags
    }

    pub fn deposit_phero(&mut self, edge: (usize, usize), tour_value: f64, tour_weight: f64, p_rate: f64) {
        let value = (tour_value*p_rate) / tour_weight;
        self.tau.add_to_edge(edge.0, edge.1, value);
    }
}

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

#[cfg(test)]
mod test  {
    use std::vec;

    use super::*;
    #[test]
    fn tau() {
        let mut tao = Tau::new();
        tao.set_edge(10, 15, 100.0);
        assert_eq!(tao.get_edge(10, 15), 100.0);
        assert_eq!(tao.get_edge(10, 15), tao.get_edge(15, 10));
        tao.add_to_edge(15, 10, 100.0);
        assert_eq!(tao.get_edge(10, 15), 200.0);
    }

    #[test]
    fn path_selection() {
        let mut probabilities: Vec<f64> = vec![0.2, 0.3, 0.1, 0.4];
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