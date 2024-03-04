use serde::{Serialize};
use serde::ser::{SerializeMap, Serializer};
use serde_json::{self, Map, Value};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};

use crate::simulation::agent::Agent;
use std::{hash::Hash};
use ahash::AHashMap; // this is a bit faster than the standard HashMap
use std::fmt;
use rand::Rng;
use rand::distributions::{Distribution, WeightedIndex};
use std::result::Result;
use memory_stats::memory_stats;
use std::borrow::BorrowMut;
use crate::simulation::types::Game;

pub fn print_memory_usage() {
    if let Some(usage) = memory_stats() {
        println!("Current physical memory usage: {} GB", usage.physical_mem as f64/1e6);
        println!("Current virtual memory usage: {} GB", usage.virtual_mem as f64/1e6);
    } else {
        println!("Couldn't get the current memory usage :(");
    }
}
#[derive(Clone, Debug)]
pub struct RoundState {
    pub state: AHashMap<Agent, (u32, f64, f64, f64)>, // (count, payoff, fitness, population_share)
    average_score: f64,
}

impl<'de> Deserialize<'de> for RoundState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RoundStateVisitor;

        impl<'de> Visitor<'de> for RoundStateVisitor {
            type Value = RoundState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RoundState")
            }

            fn visit_map<V>(self, mut map: V) -> Result<RoundState, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut state: Option<Vec<(Agent, (u32, f64, f64, f64))>> = None;
                let mut average_score = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "state" => {
                            if state.is_some() {
                                return Err(de::Error::duplicate_field("state"));
                            }
                            // Deserialize as a vector of tuples
                            state = Some(map.next_value::<Vec<(Agent, (u32, f64, f64, f64))>>()?);
                        }
                        "average_score" => {
                            if average_score.is_some() {
                                return Err(de::Error::duplicate_field("average_score"));
                            }
                            average_score = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                let state = state.ok_or_else(|| de::Error::missing_field("state"))?;
                let average_score = average_score.ok_or_else(|| de::Error::missing_field("average_score"))?;

                // Convert the Vec<(Agent, (u32, f64, f64, f64))> into AHashMap<Agent, (u32, f64, f64, f64)>
                let state_map = state.into_iter().collect::<AHashMap<Agent, (u32, f64, f64, f64)>>();

                Ok(RoundState { state: state_map, average_score })
            }
        }

        const FIELDS: &'static [&'static str] = &["state", "average_score"];
        deserializer.deserialize_struct("RoundState", FIELDS, RoundStateVisitor)
    }
}

impl Serialize for RoundState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = Map::new();
        for (agent, (count, payoff, fitness, population_share)) in &self.state {
            // Ensure Agent has a method `unique_id` that returns a unique string identifier
            let key = agent.to_json(); // This should be a simple string identifier, not JSON
            let value = serde_json::to_value((payoff, fitness, population_share)).unwrap();
            map.insert(key, value);
        }
        let mut state = serializer.serialize_map(Some(self.state.len()))?;
        state.serialize_entry("state", &map)?;
        state.serialize_entry("average_score", &self.average_score)?;
        state.end()
    }
}

impl RoundState {
    pub fn new() -> RoundState {
        RoundState {
            state: AHashMap::new(),
            average_score: 1.0,
        }
    }   

    pub fn from(state: AHashMap<Agent, (u32,f64, f64, f64)>, average_score: f64) -> RoundState {
        RoundState {
            state: state,
            average_score: average_score,
        }
    }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(&self).unwrap()
    }

    pub fn save_to_file(&self, filename: &str) {
        let json = self.to_json();
        std::fs::write(filename, json).expect("Unable to write file");
    }
}

#[derive(Clone, Debug)]
pub struct GamePlay {
    game: Game,
    pub agents: AHashMap<Agent, (u32, f64)>,
    pub gamestate: Vec<RoundState>, //initialize as None

    // note these probabilities might be conditioned on the agent in some way so 
    // another data structure might be needed
    pub p_p : f64, // proba of PointMutation
    pub p_d : f64, // proba of GeneDuplication
    pub p_s : f64, // proba of SplitMutation
    pub d : f64, // death rate
    n: u32, // population size
    debug: bool,
}

impl GamePlay {
    pub fn new(agents: Vec<Agent>, debug : bool, game : Option<Game>) -> GamePlay {
        let agents = GamePlay::get_unique_genomes(agents);

        //we default to the prisoners dilemma game if no game is provided
        let game = match game {
            Some(game) => game,
            None => Game::new("prisoners_dilemma".to_string()).expect("Error in creating game"),
        };

        GamePlay {
            game: game,
            agents: agents,
            gamestate: Vec::new(),
            debug: debug,
            p_p: 2e-5, // PointMutation proba from paper
            p_d: 1e-5, // GeneDuplication proba from paper
            p_s: 1e-5, // SplitMutation proba from paper
            d: 0.1,
            n: 1000,
        }
    }

    pub fn random_init(n: u32, m: u32, debug : bool) -> GamePlay {
        let mut agents: Vec<Agent> = Vec::new();
        for _ in 0..n {
            agents.push(Agent::random_init(m));
        }

        let game = Game::new("prisoners_dilemma".to_string()).expect("Error in creating game");
        GamePlay::new(agents, debug, Some(game))
    } 

    pub fn run(&mut self, rounds: u32) -> Result<(), String> {
        let mut state : Vec<RoundState> = Vec::new();
        for _ in 0..rounds {

            let time = std::time::Instant::now();
            let next_proba_distb = self.compute_round(self.agents.clone());
            //do some sort of sampling of the agents
            let agents = self.sample_and_apply_mutations(next_proba_distb, self.n);
            
            if agents.keys().len() < 2 {
                return Err(format!("Population size has converged to 0 or 1 agents, stopping simulation. agent left: {:?}", self.agents));
            }
            self.agents = agents;
            if self.debug {
                print_memory_usage();
                println!("Time taken for round: {:?}", time.elapsed());
            }
        }
        Ok(())
    }

    pub fn sample_and_apply_mutations(
        &self, next_proba_distb: AHashMap<Agent, f64>, n: u32
    ) -> AHashMap<Agent, (u32, f64)> {
        let mut rng = rand::thread_rng();
        let mut new_agents : Vec<Agent> = Vec::new();
        // Assuming Agent implements Clone + PartialEq
        let prev_agents: Vec<Agent> = next_proba_distb.keys().cloned().collect();
        let weights: Vec<f64> = next_proba_distb.values().cloned().collect();

        println!("weights: {:?}", weights);
        let dist = WeightedIndex::new(&weights).unwrap();

        for _ in 0..n {
            let mut sampled_agent = prev_agents[dist.sample(&mut rng)].clone();
            new_agents.push(sampled_agent);
        }    

        if self.debug {
            println!("unique agents before mutation: {:?}", GamePlay::get_unique_genomes(new_agents.clone()).len());
        }
        
        for i in 0..n {
            let old_agent = new_agents[i as usize].clone();
            let sampled_agent = new_agents[i as usize].borrow_mut();
            let mutated = sampled_agent.mutate(self.p_p, self.p_d, self.p_s); 
            //TODO if mutated, split from old key and add new entry
        }
        
        if self.debug {
            println!("unique agents after mutation: {:?}", GamePlay::get_unique_genomes(new_agents.clone()).len());
        }

        let new_generation = GamePlay::get_unique_genomes(new_agents.clone());
        if self.debug {
            println!("total agents: {:?}", new_generation.values().map(|(count, _)| count).sum::<u32>());
            println!("new agents count: {:?}", new_agents.len());
        }

        //checking if mutations are working
        new_generation
    }

    fn get_unique_genomes(agents : Vec<Agent>) -> AHashMap<Agent, (u32, f64)>{
        let mut agent_map: AHashMap<Agent, (u32, f64)> = AHashMap::new();
        let total_count = agents.len(); 
        for (idx, agent) in agents.iter().enumerate() {
            if !agent_map.contains_key(agent) {
                agent_map.insert(agent.clone(), (1, 1.0/total_count as f64) );
            } else {
                //we increment the share of the agent
                let (count, share) = agent_map.get_mut(&agent).unwrap();
                *count += 1;
                *share += 1.0/total_count as f64;
            }
        }
        agent_map
    }

    fn compute_round(&self, mut agents : AHashMap<Agent, (u32, f64)>) -> AHashMap<Agent, f64> {
        //computes the state of the game after one round and return proba_distb of agents in next round
        if agents.len() < 2 {
            panic!("Population size has converged to 0 or 1 agents, stopping simulation. agent left: {:?}", agents);
        }

        let mut state: AHashMap<Agent, (u32, f64, f64, [bool; 2])> = AHashMap::new();
        let mut average_score = 0.0; // Placeholder for action history updates
        let mut action= [false, false]; 
        // note has to be declared as the compiler doesnt understand that it will be initialized in the loop 
        // as the loop is quanarentted to run at least once as we dont allow agents to be less than 2
        let mut initialized = false;
        

        for i in 0..agents.len(){
            let mut payoff1 = 0.0;

            let (agent1, (count1, share1)) = agents.iter().nth(i).unwrap();
            for j in 0..agents.len(){
                if i == j {
                    continue;
                } 
                let (agent2, (count2, share2)) = agents.iter().nth(j).unwrap();

                let action1 = agent1.map_history_to_action();
                let action2 = agent2.map_history_to_action();

                let payoff = self.game.get_payoff((action1, action2));
                //we add the payoff to the agents, can be done more efficiently for sure
                println!("agent one took action: {:?}", action1);
                println!("agent two took action: {:?}", action2);
                action = [action1, action2];
                
                if self.debug {
                    println!("Agent1 took action: {:?} Agent2 took action: {:?}",  action1, action2);
                    println!("payofs: {:?}", payoff);
                }
                
                payoff1 += payoff.0 * share2.clone(); // increment by the payoff against agent2 times agent2's share in the population
            }
            average_score += payoff1 * *share1; //average score is payoff times share of agent in population
            state.insert(agent1.clone(), (count1.clone(), payoff1, share1.clone(), action));
        }

        //we need to have the average in order to calculate the fitness
        let mut final_state : AHashMap<Agent, (u32, f64, f64, f64)> = AHashMap::new(); 
        for (agent, (count, payoff, population_share, action)) in state.iter_mut(){
            let fitness = *payoff - average_score;
            let mut temp_agent = agent.clone();
            temp_agent.add_memory(*action); 
            final_state.insert(temp_agent.clone(), (*count, *payoff, fitness, *population_share));
        }

        let round_state = RoundState {
            state: final_state,
            average_score: average_score,
        };
        
        let next_proba_distb = self.update_population_distribution( &round_state);
        next_proba_distb
    }

    /* 
    pub fn get_proba_q_lower_ij(&self, agent: &Agent, agent_share : f64, n : u32, j: u32) -> f64 {
        // not complete yet
        // three probabilities to take into account are p_p, p_d, p_s

        // probability of mutation from point mutation, is total_prob*len(genome) 
        // since this mutation is uniformly distributed
        let q_p_i = self.p_p * (1.0/agent.genome.len() as f64);
        
        // probability of mutation from gene duplication 
        let q_d_i = 0.0; // TODO, dont know how to implement this, yet
        // probability of mutation from split mutation, is total_prob*1/2 as each half is equally likely
        let q_s_i = self.p_d * (1.0/2.0); 

        let total_prob = q_p_i + q_d_i + q_s_i;
        return total_prob;
       
    } */

    /* pub fn compute_m_i(&self, agent: &Agent, round_state: &RoundState) -> f64 {
        // not complete
        let mut m_i = 0.0;
        let n = self.agents.len() as f64; // Total number of agents

        // Iterate over all possible mutations to the current agent
        for (other_agent, (_, _, _, other_population_share)) in round_state.state.iter() {
            // Calculate Q_ij for the mutation from other_agent to agent
            let q_ij = self.get_proba_q_lower_ij(other_agent, *other_population_share, n as u32, 1); // Assuming j = 1 for simplicity
            let q_ji = self.get_proba_q_lower_ij(agent, *other_population_share, n as u32, 1); // Assuming i = 1 for simplicity

            // Calculate the stochastic term Q_ij - Q_ji
            let delta_q = q_ij - q_ji;

            // Add the contribution of this mutation to m_i
            m_i += delta_q;
        }

        // Normalize m_i by the population size N
        m_i /= n;

        m_i
    } */

    pub fn update_population_distribution(&self, last_roundState: &RoundState) -> AHashMap<Agent, f64> {
        let mut proba_distb: AHashMap<Agent, f64> = AHashMap::new();
        let mut counts : Vec<u32> = Vec::new();
        let mut total_next_proba = 0.0;

        for (agent, (count, payoff, fitness, population_share)) in last_roundState.state.iter() {
            let x = 1.0 - last_roundState.state.iter().map(
                    |(agent2, (_, payoff2, _, population_share2))| 
                    if agent == agent2 {0.0} else {(payoff2*population_share2)/payoff }
                ).sum::<f64>();
            
            println!("agent: {:?}", agent);
            println!("payoff: {:?}", payoff);
            println!("fitness: {:?}", fitness);
            println!("population_share: {:?}", population_share);

            let next_proba = payoff * self.d * population_share * x;

            if (next_proba > (1.0 / self.n as f64)) {
                //we only add the agent to the next round if the probability is greater than 1/n
                proba_distb.insert(agent.clone(), next_proba);
                total_next_proba += next_proba;
                counts.push(*count);
            } else {
                println!("excluding with proba: {:?}", next_proba);
            }
        }

        if total_next_proba > 0.0 {
            for (agent, proba) in proba_distb.iter_mut() {
                *proba /= total_next_proba;
            }
        } else {
            panic!("Total next probability is 0, cannot normalize distribution");
        }

        // Check if the probability distribution sums to 1
        let proba_sum: f64 = std::iter::zip(proba_distb.values(), counts).
            map(|(proba, _ )| proba).sum();
            
        if (1.0 - proba_sum).abs() > 1e-5 {
            panic!("Proba distribution does not sum to 1 got: {:?}", proba_sum);
        }
    
        proba_distb
    }

}