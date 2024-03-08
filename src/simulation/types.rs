use serde::{Serialize, Deserialize};
use ahash::AHashMap;
use crate::simulation::agent::Agent;

#[derive(Debug, Clone)]
pub struct GameBoard {
    payoff_matrix: AHashMap<Vec<bool>, Vec<f64>>,
    pub n_players: u32,
}

impl GameBoard {
    pub fn new(game_name: String, n_players : u32) -> Result<GameBoard, String> {
        
        let mut payoff_matrix : AHashMap<Vec<bool>, Vec<f64>> = AHashMap::new();
        match game_name.as_str() {
            "prisoners_dilemma" => {
                match n_players  {
                2 => {
                    payoff_matrix.insert(vec![true, true], vec![5.0, 10.0]); // agent 1 cooperates, agent 2 cooperates
                    payoff_matrix.insert(vec![true, false], vec![1.0, 15.0]); // agent 1 cooperates, agent 2 defects
                    payoff_matrix.insert(vec![false, true], vec![10.0, 10.0]); // agent 1 defects, agent 2 cooperates
                    payoff_matrix.insert(vec![false, false], vec![0.0, 5.0]);  // agent 1 defects, agent 2 defects
                }, 
                3 => {
                    // All cooperate
                    payoff_matrix.insert(vec![true, true, true], vec![5.0, 5.0, 5.0]);
                    // Two cooperate, one defects
                    payoff_matrix.insert(vec![true, true, false], vec![3.0, 3.0, 10.0]);
                    payoff_matrix.insert(vec![true, false, true], vec![3.0, 10.0, 3.0]);
                    payoff_matrix.insert(vec![false, true, true], vec![10.0, 3.0, 3.0]);
                    // One cooperates, two defect
                    payoff_matrix.insert(vec![true, false, false], vec![1.0, 0.0, 0.0]);
                    payoff_matrix.insert(vec![false, true, false], vec![0.0, 1.0, 0.0]);
                    payoff_matrix.insert(vec![false, false, true], vec![0.0, 0.0, 1.0]);
                    // All defect
                    payoff_matrix.insert(vec![false, false, false], vec![0.0, 0.0, 0.0]);
                }

                _ => return Err(format!("Invalid number of players: {} not implemted yet", n_players)),
            };
            },
            _ => return Err(format!("Invalid game name: {} not implemted yet", game_name)),
        }
        
        Ok(GameBoard {
            payoff_matrix: payoff_matrix,
            n_players: n_players,
        })
    }

    pub fn get_payoff(&self, action: &Vec<bool>) -> Vec<f64> {

        if action.len() != self.n_players as usize {
            panic!("Invalid action length: {} expected {}", action.len(), self.n_players);
        }

        match self.payoff_matrix.get(action) {
            Some(payoff) => payoff.clone(),
            None => panic!("Invalid action: {:?}", action),
        }
    }


}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRoundData {
    pub count : u32, 
    pub score : f64,
    pub fitness : f64,
    pub population_share : f64
}

impl AgentRoundData {
    pub fn from(count: u32, score : f64, fitness : f64, population_share : f64) -> AgentRoundData {
        AgentRoundData {
            count: count,
            score: score,
            fitness: fitness,
            population_share: population_share
        }
    }
}

///this holds features like count and population share which are useful to have precomputed for each round
/// it is used in the hashmap with agents IE AHashMap<Agent, AgentMetaData>
/// # Variables:
///    pub count: u32 - the number of agents with the same genome
///     pub population_share: f64 - the share of the population with the same genome
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentMetaData {
    pub count : u32,
    pub population_share : f64,
}

impl AgentMetaData {
    pub fn new() -> AgentMetaData {
        AgentMetaData {
            count: 0,
            population_share: 0.0,
        }
    }
}

/// This holds the state of the game at a given round
/// # Variables:
///     pub round_number: u32 - the round number
///     pub agent_data: AHashMap<Agent, AgentRoundData> - the data for each agent in the round
#[derive(Debug, Clone)]
pub struct RoundState {
    pub round_number : u32,
    pub agent_data : AHashMap<Agent, AgentRoundData>,
}

impl RoundState {
    pub fn new(round_number: u32) -> RoundState {
        RoundState {
            round_number: round_number,
            agent_data: AHashMap::new(),
        }
    }
}