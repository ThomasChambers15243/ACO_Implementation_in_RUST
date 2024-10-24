use core::fmt;
use std::collections::HashMap;
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Edge {
    pub bag_i: Box<Bag>,
    pub bag_j: Box<Bag>,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge: Bag_i {} <-> Bag_j {}", self.bag_i.number, self.bag_j.number)
    }
}

impl Edge {
    pub fn new(bag_i: Box<Bag>, bag_j: Box<Bag>) -> Self {
        if bag_i < bag_j {
            Edge { bag_i, bag_j }
        } else {
            Edge { bag_i: bag_j, bag_j: bag_i }
        }
    }

    pub fn contains(&self, bag: &Box<Bag>) -> bool {
        &self.bag_i == bag || &self.bag_j == bag
    }
}

#[derive(Debug)]
pub struct Graph {
    pub max_weight: f64,
    pub nodes: usize,
    pub graph: Box<[Bag; 100]>, 
    pub tau: HashMap<Edge, f64>,
}

impl Graph {
    pub fn construct_graph(beta: f64) -> Self {
        let (max_weight, bags) = load_data(beta);
        let nodes = bags.len();
        let graph: [Bag; 100] = bags.try_into().unwrap();

        Graph {
            max_weight,
            nodes,
            graph: Box::new(graph),
            tau: HashMap::new(),
        }
    }

    pub fn initialize_tau(&mut self) {
        let mut rng = rand::thread_rng();
        let bags = &self.graph;

        for i in 0..bags.len() {
            for j in (i + 1)..bags.len() {
                let bag_i = bags[i];
                let bag_j = bags[j];

                self.tau.insert(
                    Edge::new(Box::new(bag_i), Box::new(bag_j)),
                    rng.gen_range(1.0..2.0),
                );
            }
        }
    }

    pub fn get_availible_bags(
        &self,
        current_bag: &Bag,
        visited_bags: &Vec<Box<Bag>>,
        allowed_weight: f64,
    ) -> Vec<Box<Bag>> {
        self.graph
            .iter()
            .filter(|&bag| {
                bag.number != current_bag.number
                    && !visited_bags.contains(&Box::new(*bag))
                    && bag.weight <= allowed_weight
            })
            .map(|&bag| Box::new(bag))
            .collect()
    }

    pub fn select_path(
        &self,
        bag_i: &Bag,
        availible_bags: Vec<Box<Bag>>,
        alpha: f64,
    ) -> Option<Box<Bag>> {
        let wheel = self.create_selection_wheel(bag_i, &availible_bags, alpha);
        let choice = rand::thread_rng().gen_range(0.0..=1.0);
        availible_bags
            .into_iter()
            .zip(wheel.into_iter())
            .find(|(_, rank)| choice <= *rank)
            .map(|(bag, _)| bag)
    }

    fn create_selection_wheel(
        &self,
        bag_i: &Bag,
        availible_bags: &[Box<Bag>],
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
        bag_i: &Bag,
        bag_j: &Box<Bag>,
        availible_bags: &[Box<Bag>],
        alpha: f64,
    ) -> f64 {
        let t = self.tau[&Edge::new(Box::new(*bag_i), bag_j.clone())] * alpha;
        let h = bag_j.h;
        let sum_of_availible_bags = availible_bags
            .iter()
            .map(|bag| {
                let t = self.tau[&Edge::new(Box::new(*bag_i), bag.clone())] * alpha;
                t * bag.h
            })
            .sum::<f64>();
        (t * h) / sum_of_availible_bags
    }

    pub fn deposit_phero(&mut self, edge: Edge, tour_value: f64, best_solution: f64, p_rate: f64, decay_rate: f64) {
        if let Some(tau_val) = self.tau.get_mut(&edge) {
            *tau_val = (*tau_val * decay_rate) * (tour_value / best_solution) * p_rate;
        }
    }

    pub fn print_all_edges_for_bag(&self, bag_i: &Bag) {
        let mut count = 0.0;
        let mut avg = 0.0;
        for (edge, phero) in &self.tau {
            if edge.contains(&Box::new(*bag_i)) {
                println!("Edge: {}, Phero: {}", edge, phero);
                avg += phero;
                count += 1.0;
            }
        }
        println!("Average Phero: {}", avg / count);
    }
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
