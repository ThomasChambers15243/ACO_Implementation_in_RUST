use core::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
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
    pub graph: [Bag; 100], 
    pub tau: [[f64;100];100],
}

impl Graph {
    pub fn construct_graph(beta: f64) -> Self {
        let (max_weight, bags) = load_data(beta);
        let nodes = bags.len();
        let graph: [Bag; 100] = bags.try_into().unwrap();        

        Graph {
            max_weight,
            nodes,
            graph,
            tau: [[0.0; 100]; 100],
        }
    }

    pub fn initialize_tau(&mut self) {
        let mut rng = rand::thread_rng();
        let bags = &self.graph;

        for i in 0..bags.len() {
            for j in 0..bags.len() {
                if i != j {
                    self.tau[i][j] = rng.gen_range(0.1..1.0);
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
        let wheel = self.create_selection_wheel(bag_i, availible_bags, alpha);
        let choice = rand::thread_rng().gen_range(0.0..=1.0);
        availible_bags
            .iter()
            .zip(wheel.iter())
            .find(|(_, &rank)| choice <= rank)
            .map(|(bag, _)| *bag)
    }

    fn create_selection_wheel(
        &self,
        bag_i: &usize,
        availible_bags: &Vec<usize>,
        alpha: f64,
    ) -> Vec<f64> {        
        let probabilities: Vec<f64> = availible_bags
            .iter()
            .map(|bag| self.calculate_edge_probability(bag_i, bag, availible_bags, alpha))
            .collect();
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
        let t = self.tau[*bag_i][*bag_j] * alpha;
        let h = self.graph[*bag_j].h;
        let sum_of_availible_bags = availible_bags
            .iter()
            .map(|bag| {
                let t = self.tau[*bag_i][*bag] * alpha;
                t * self.graph[*bag].h
            })
            .sum::<f64>();
        (t * h) / sum_of_availible_bags
    }

    pub fn deposit_phero(&mut self, edge: (usize, usize), tour_value: f64, best_solution: f64, p_rate: f64, decay_rate: f64) {
        let tau_val = self.tau[edge.0][edge.1];
        self.tau[edge.0][edge.1] = (tau_val * decay_rate) * (tour_value / best_solution) * p_rate;
        
    }

    // pub fn print_all_edges_for_bag(&self, bag_i: &Bag) {
    //     let mut count = 0.0;
    //     let mut avg = 0.0;
    //     for (edge, phero) in &self.tau {
    //         if edge.contains(&Box::new(*bag_i)) {
    //             println!("Edge: {}, Phero: {}", edge, phero);
    //             avg += phero;
    //             count += 1.0;
    //         }
    //     }
    //     println!("Average Phero: {}", avg / count);
    // }
}

fn load_data(beta: f64) -> (f64, Vec<Bag>) {
    let data = fs::read_to_string("src\\BankProblem.txt").expect("Unable to read file");

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
                h: ratio * beta,
            });
            number += 1;
        }
    }

    (
        split_data.remove(0).strip_prefix("security van capacity: ").unwrap().parse().unwrap(),
        bags,
    )
}
