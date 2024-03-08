use serde::{Deserialize, Serialize};
use serde::ser::{SerializeMap, Serializer, SerializeStruct, SerializeSeq};
use serde_json::{self, Map, Value};
use serde::de::{self, Deserializer, MapAccess, Visitor};
use itertools::Itertools;
use approx::assert_abs_diff_eq;
use rayon::prelude::*;
use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;
use std::collections::HashMap;

use crate::simulation::{
    agent::Agent,
    types::{AgentRoundData, RoundState, GameBoard}
};
use std::{hash::Hash};
use super::types::AgentMetaData; // this is a bit faster than the standard HashMap

#[derive(Clone, Debug, Deserialize)]
pub struct Game {
    pub rounds : Vec<RoundState>,
    pub game_board : GameBoard,
    pub is_test : bool,
    pub debug : bool, 
    pub d: f64,
    pub p_p: f64,
    pub p_d: f64,
    pub p_r: f64,
}

impl Serialize for Game {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        let mut state = serializer.serialize_struct("Game", 8)?;
        state.serialize_field("rounds", &self.rounds)?;
        state.serialize_field("game_board", &self.game_board)?;
        state.serialize_field("is_test", &self.is_test)?;
        state.serialize_field("debug", &self.debug)?;
        state.serialize_field("d", &self.d)?;
        state.serialize_field("p_p", &self.p_p)?;
        state.serialize_field("p_d", &self.p_d)?;
        state.serialize_field("p_r", &self.p_r)?;
        state.end()
    }

}


impl Game {
    pub fn new(
        game_board: GameBoard, 
        is_test : bool, 
        debug : bool,
    ) -> Result<Game, String> {
        
        Ok(Game {
            rounds: Vec::new(),
            game_board: game_board,
            is_test : is_test,
            debug : debug,
            d: 0.001,
            p_p: 2e-5,
            p_d: 1e-5,
            p_r: 1e-5,
        })
    }

    pub fn dump_round_state_to_json(&self, file_path : String){
        serde_json::to_writer(
            std::fs::File::create(file_path).unwrap(), &self.rounds
        ).unwrap();
    }

    pub fn agents_to_hashmap(agents : &Vec<Agent>)-> HashMap<Agent, AgentMetaData> {
        let mut agents_map : HashMap<Agent, AgentMetaData> = HashMap::new();
        
        for agent in agents {
            if agents_map.contains_key(agent) {
                let agent_metadata = agents_map.get_mut(agent).unwrap();
                agent_metadata.count += 1;
            } else {
                let mut agent_metadata = AgentMetaData::new();
                agent_metadata.count = 1;
                agents_map.insert(agent.clone(), agent_metadata);
            }
        }
        // we compute population share
        let agents_len = agents.len();
        agents_map.iter_mut().for_each(|(_ , metadata)| {
            metadata.population_share = metadata.count as f64 / agents_len as f64;
        });

        agents_map
    }

    pub fn run(&mut self, n_rounds : u32, agents : Vec<Agent>) {

        let mut agents_map = Game::agents_to_hashmap(&agents);  
        for i in 0..n_rounds {
            let round_state = self.play_round(i, agents_map.clone(), agents.len() as u32);
            self.rounds.push(round_state.clone());
            agents_map = self.sample_new_agents(&round_state, agents.len() as u32); // Updated without redeclaration
            if agents_map.keys().len() < 2 {
                println!("game halted at timestep: {:?} as there is only one agent left", i);
                break;
            }

        }
    }

    /// this is a nonlinear function that makes the value positive
    pub fn make_positive(&self, value : f64) -> f64{
        //we use exponential function to make the value positive
        1.0 + (-value).exp()
        
    }

    pub fn compute_next_probability(&self, roundData : &RoundState) -> HashMap<Agent, f64> {
        let mut outcome_probabilities : HashMap<Agent, f64> = HashMap::new();
        let mut total_probability = 0.0;
        
        for (i, (agent, agent_data)) 
            in roundData.agent_data.iter().enumerate() {
            
            let first_factor = self.d * self.make_positive(agent_data.fitness)*agent_data.population_share;
            
            let second_factor : f64;
            if agent_data.score != 0.0 {
                second_factor = 1.0 - roundData.agent_data
                    .iter().enumerate().
                    map(|(j, (agent, inner_agent_data))|
                        if i != 0 {
                            (inner_agent_data.score * inner_agent_data.population_share)/ agent_data.score
                        } else {
                            0.0
                        }
                    ).sum::<f64>();   
            } else {
                second_factor = 1.0;
            }                 
                
            let second_factor = self.make_positive(second_factor);
            //check is not NaN
            if first_factor.is_nan() {
                panic!("Nan value detected in probability computation for first factor");
            }
            if second_factor.is_nan() {
                panic!("Nan value detected in probability computation for second factor");
            }
            let probability = first_factor * second_factor;
            total_probability += probability;
            
            outcome_probabilities.insert(agent.clone(), probability);
        }

        
        // Test if the sum of the probabilities is 1
        let sum : f64 = outcome_probabilities.iter().map(|(_, &prob)| prob).sum();
        
        if sum != 0.0 {
            for probability in outcome_probabilities.values_mut() {
                *probability /= sum;
            }
        }

        let corrected_sum: f64 = outcome_probabilities.values().sum();
        if corrected_sum != 1.0 {
            let correction_factor = 1.0 / corrected_sum;
            for probability in outcome_probabilities.values_mut() {
                *probability *= correction_factor;
            }
        }

        let sum : f64 = outcome_probabilities.iter().map(|(_, &prob)| prob).sum();
        
        assert_abs_diff_eq!(sum, 1.0, epsilon = 0.0001);
        for (_, prob) in outcome_probabilities.iter() {
            assert!(*prob >= 0.0); // proba should be positive
            assert!(*prob <= 1.0); // proba should be less than 1
        }
        
        outcome_probabilities
    }

    /// mutates the agents in place
    /// TODO: this could be optimized further by using a parallel iterator
    /// and by using a dictionary to store the agents
    pub fn apply_mutations(&self, agents : &mut Vec<Agent>) {

        agents.par_iter_mut().for_each(|agent| {  
            agent.mutate(self.p_p, self.p_d, self.p_r);
            // Mutation logic is applied directly to each agent in the vector,
            // so there's no need to return a new vector.
            // The mutate method should modify the agent in place.
        });
    }

    /// this function first samples new agents based on the fitness from last round data
    pub fn sample_new_agents(
        &mut self, round_data : &RoundState, n_agents : u32
    ) -> HashMap<Agent, AgentMetaData>{
        if round_data.agent_data.len() == 0 {
            panic!("No agents in round data map is empty");
        }
        let probability_distribution = self.compute_next_probability(round_data);
        // Convert the probability distribution into a format suitable for sampling
        let agents: Vec<Agent> = probability_distribution.keys().cloned().collect();
        let probabilities: Vec<f64> = probability_distribution.values().cloned().collect();
        if self.debug {
            println!("Probabilities: {:?}", probabilities);
        }
        let dist = WeightedIndex::new(&probabilities).unwrap();
    
        // Sample in parallel
        let mut new_agents: Vec<Agent> = (0..n_agents).into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                agents[dist.sample(&mut rng)].clone()
            })
            .collect();

        self.apply_mutations(&mut new_agents); // we modify the agents in place 
        let output = Self::agents_to_hashmap(&new_agents);
        return output;
    }

    /// this is the core function of the game, 
    /// it computes the score for each agent along with 
    /// fitness over all possible interactions
    fn play_round(
        &mut self, 
        round_number : u32, 
        agents : HashMap<Agent, AgentMetaData>,
        total_players : u32
    ) -> RoundState {
        let combinations = self.enumerate_combinations(
            &agents.keys().cloned().collect()   
        );

        let mut inter_mediate_compute : HashMap<Agent,f64> = HashMap::new();
        // this loop computes the score for all the agents
       
        for (agent_combinations, actions, scores) in combinations { 
            // we compute the scores for each agent and adds them to a temporary dictionary/hashmap
            for (i, agent) in agent_combinations.iter().enumerate() {
                // this computes the score for agent i by weighting the score of the ith agent by 
                // forall j in 0 ...n  where j != n do scores[i] * agents[j].population_share 
                let mut agent_combinations_clone = agent_combinations.clone();
                agent_combinations_clone.remove(i); // we remove the agent itself
                let mut score = 0.0;
                for agent in agent_combinations_clone.iter() {
                    // we weight the score by the population share of the opponents
                    score += scores[i] * agents.get(agent).unwrap().population_share;
                }

                if inter_mediate_compute.contains_key(agent) {
                    let old_score = inter_mediate_compute.get_mut(agent).unwrap();
                    *old_score += score;
                } else {
                    let cloned_agent = agent.clone();
                    inter_mediate_compute.insert(cloned_agent, score);
                }    
            }    
        }

        let average_score = inter_mediate_compute.iter().
            map(
                |(agent, score)| 
                score * agents.get(agent).unwrap().population_share as f64 // mutliply agent score by its share of the population
            ).sum::<f64>() / total_players as f64;

        let agent_data : HashMap<Agent, AgentRoundData> = inter_mediate_compute.iter().map(
            |(agent, score)| {
            let agent_metadata = agents.get(agent).unwrap();
            let (count, population_share) = (agent_metadata.count, agent_metadata.population_share);
            let fitness = score -average_score ;
            let cloned_agent = agent.clone();
            (cloned_agent, AgentRoundData::from(count, *score, fitness, population_share))
        }).collect();

        return RoundState {
            round_number: round_number,
            agent_data: agent_data,
        }

    }

    /// this enumerates all possible combinations of agents and their actions, score and against each other
    /// # Args:
    ///     agents: Vec<Agent> - the agents to be used in the game
    ///# Returns:
    ///     Vec<(Vec<Agent>, Vec<bool>, Vec<f64>)> - a vector of tuples 
    ///     Vec<Agent> - the agents in the combination
    ///     Vec<bool> - the actions of the agents in the combination where Agent[i] takes action[i]
    ///     Vec<f64> - the score of the agents in the combination where Agent[i] gets score[i]
    fn enumerate_combinations(
        &self, agents: &Vec<Agent>
    ) -> Vec<(Vec<Agent>, Vec<bool>, Vec<f64>)> 
    {   
        let mut results = Vec::new();
        // Generate all unique combinations of agents of size total
        let n_players = self.game_board.n_players as usize;

        for combination in agents.iter().sorted().combinations(n_players) {
            let agents_combination: Vec<Agent> = combination.iter().
                map(|agent| *agent).cloned().collect();
            let actions: Vec<bool> = agents_combination.iter().
                map(|agent| agent.get_action()).collect();
            let score = self.game_board.get_payoff(&actions);
            results.push((agents_combination, actions, score));
        }
        results
    }


}