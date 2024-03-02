/* 
Notes:
noise with probability p the action is opposite of the intended one 
Time t >> 1/(2p)

Population dynamics of n agents
everyone plays the prisoners dilemma with everyone else
*/

// implementation not finished
use serde::{Serialize, Deserialize};
use crate::simulation::agent::Agent;
use std::{collections::HashMap, hash::Hash};

struct RoundState {
    state: HashMap<Agent, (f64, f64, f64)>, // (payoff, fitness, population_share)
    average_score: f64,
}


pub struct GameState {
    state: Vec<RoundState>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            state: vec![],
        }
    }

    pub fn to_json(&self) -> String {
        
    }
}

pub struct Game {
    payoff_map: HashMap<(bool, bool), (f64, f64)>,
    agents: Vec<Agent>,
    state: Vec<RoundState>, //initialize as None
}

impl Game {
    pub fn new(agents: Vec<Agent>) -> Game {
        let mut payoff_map = HashMap::new();
        // we are using the prisoners dilemma as an example
        payoff_map.insert((true, true), (1.0, 1.0));
        payoff_map.insert((true, false), (0.0, 3.0));
        payoff_map.insert((false, true), (3.0, 0.0));
        payoff_map.insert((false, false), (2.0, 2.0));
        Game {
            payoff_map: payoff_map,
            agents: agents,
            state: vec![],
        }
    }

    pub fn get_unique_genomes(&self) -> HashMap<Agent, (Vec<u32>, f64)>{
        let mut agent_map: HashMap<Agent, (Vec<u32>, f64)> = HashMap::new();
        let total_count = self.agents.len(); 
        for (idx, agent) in self.agents.iter().enumerate() {
            if !agent_map.contains_key(agent) {
                agent_map.insert(agent.clone(), (vec![idx as u32], 1.0/total_count as f64) );
            } else {
                //we increment the share of the agent
                let (idxs, share) = agent_map.get_mut(&agent).unwrap();
                idxs.push(idx as u32);
                *share += 1.0/total_count as f64;
            }
        }
        return agent_map;
    }

    fn compute_round(&self) -> (HashMap<Agent, f64>, RoundState) {
        //computes the state of the game after one round and return proba_distb of agents in next round

        let unique_agents = self.get_unique_genomes();
        let mut state: HashMap<Agent, (f64, f64)> = HashMap::new();
        let mut proba_distb = vec![0.0; self.agents.len()]; // proba_distb must sum to 1
        let mut average_score = 0.0;

        for (agent1, (idxs1, share1)) in unique_agents.iter(){
            let mut payoff1 = 0.0;
            for (agent2, (idxs2, share2)) in unique_agents.iter(){
                if agent1 == agent2 {
                    continue;
                } 
                
                let action1 = agent1.map_history_to_action();
                let action2 = agent2.map_history_to_action();
                let payoff = self.payoff_map.get(&(action1, action2)).unwrap();

                //we add the payoff to the agents, can be done more efficiently for sure
                payoff1 += payoff.0 * share2; //payoff against opponent X share of oppponents in population
                
            }

            average_score += payoff1 * share1; //average score is payoff times share of agent in population
            state.insert(agent1.clone(), (payoff1, share1.clone()));
        }

        let mut final_state : HashMap<Agent, (f64, f64, f64)> = HashMap::new(); 
        for (agent, (payoff, population_share)) in state.iter(){
            let fitness = payoff - average_score;
            final_state.insert(agent.clone(), (*payoff, fitness, *population_share));
        }

        let round_state = RoundState {
            state: final_state,
            average_score: average_score,
        };


        let next_proba_distb = self.update_population_distribution(0.1, &round_state);
        
        return (next_proba_distb , round_state);
    }

    fn update_population_distribution(&self, d: f64, state: &RoundState) -> HashMap<Agent, f64>{
        //todo
        let mut proba_distb = HashMap::new();
        for (agent, (payoff, fitness, population_share)) in state.state.iter(){
            
            let mut factor = 0.0;
            for (agent2, (payoff2, fitness2, population_share2)) in state.state.iter(){
                factor += (population_share2*payoff2)/payoff;
            }
            let next_population_distb = d*payoff * (1.0 - factor);
            proba_distb.insert(agent.clone(), next_population_distb);
        }

        return proba_distb;
    }

}


