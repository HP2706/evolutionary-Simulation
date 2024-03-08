use serde::{Serialize};
use serde::ser::{SerializeMap, Serializer};
use serde_json::{self, Map, Value};
use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use itertools::Itertools;

use crate::simulation::{
    agent::Agent,
    types::{AgentRoundData, RoundState, GameBoard}
};
use std::{hash::Hash};
use ahash::AHashMap;

use super::types::AgentMetaData; // this is a bit faster than the standard HashMap



pub struct Game {
    pub rounds : Vec<RoundState>,
    pub game_board : GameBoard,
    pub is_test : bool,
}

impl Game {
    pub fn new(
        game_board: GameBoard, 
        is_test : bool, 
    ) -> Result<Game, String> {
        
        Ok(Game {
            rounds: Vec::new(),
            game_board: game_board,
            is_test : is_test
        })
    }

    pub fn agents_to_hashmap(agents : &Vec<Agent>)-> AHashMap<Agent, AgentMetaData> {
        let mut agents_map : AHashMap<Agent, AgentMetaData> = AHashMap::new();
        
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

        let agents_map = Game::agents_to_hashmap(&agents);  
        for i in 0..n_rounds {
            let round_state = self.play_round(i, agents_map.clone(), agents.len() as u32);
            //println!("Roundstate : {:?}", round_state);
            self.rounds.push(round_state);
            
            //do sampling based on RoundState to generate new Vec<Agent> 
            //of the same size although abstraction should allow decrease/increase in agent size
            //assign new hashmap to agents_map
            //repeat
        }
    }

    pub fn sample_new_agents(&mut self, roundData : &RoundState){
        //TODO
    }

    /// this is the core function of the game, 
    /// it computes the score for each agent along with 
    /// fitness over all possible interactions
    fn play_round(
        &mut self, 
        round_number : u32, 
        agents : AHashMap<Agent, AgentMetaData>,
        total_players : u32
    ) -> RoundState {
        let combinations = self.enumerate_combinations(
            &agents.keys().cloned().collect()   
        );

        let mut inter_mediate_compute : AHashMap<Agent,f64> = AHashMap::new();
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

        let agent_data : AHashMap<Agent, AgentRoundData> = inter_mediate_compute.iter().map(
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