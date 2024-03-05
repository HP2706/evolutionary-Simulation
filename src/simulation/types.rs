use serde::{Serialize, Deserialize};
use ahash::AHashMap;

#[derive(Debug, Clone)]
pub struct Game {
    payoff_matrix: AHashMap<(bool, bool), (f64, f64)>,
}


impl Game {
    pub fn new(game_name: String) -> Result<Game, String> {
        
        let mut payoff_matrix = AHashMap::new();
        match game_name.as_str() {
            "prisoners_dilemma" => {
                payoff_matrix.insert((true, true), (5.0, 10.0)); // agent 1 cooperates, agent 2 cooperates
                payoff_matrix.insert((true, false), (1.0, 15.0)); // agent 1 cooperates, agent 2 defects
                payoff_matrix.insert((false, true), (0.0, 0.0)); // agent 1 defects, agent 2 cooperates
                payoff_matrix.insert((false, false), (0.0, 5.0));  // agent 1 defects, agent 2 defects
            },
            _ => return Err(format!("Invalid game name: {}", game_name)),
        }
        
        Ok(Game {
            payoff_matrix: payoff_matrix,
        })
    }

    pub fn get_payoff(&self, action: (bool, bool)) -> (f64, f64) {
        match self.payoff_matrix.get(&action) {
            Some(payoff) => *payoff,
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