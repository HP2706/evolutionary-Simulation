/* 
/* 
Notes:
noise with probability p the action is opposite of the intended one 
Time t >> 1/(2p)

Population dynamics of n agents
everyone plays the prisoners dilemma with everyone else
*/

// implementation not finished

use crate::simulation::agent::Agent;
use std::collections::HashMap;

struct RoundState{
    payoffs: Vec<f64>,
    actions: Vec<bool>, 
    proba_distb: Vec<f64>, // must sum to 1
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


    pub fn get_unique_genomes(&self) -> HashMap<Agent, Vec<u32>>{
        let mut agent_map = HashMap::new();
        for (idx, agent) in self.agents.iter().enumerate() {
            if !agent_map.contains_key(agent) {
                agent_map.insert(agent.clone(), vec![idx as u32]);
            } else {
                agent_map.get_mut(&agent).unwrap().push(idx as u32);
            }
            
        }
        return agent_map;
    }

    fn compute_round(&self){
        let unique_agents = self.get_unique_genomes();
        let mut payoffs : Vec<f64> = vec![0.0; self.agents.len()];
        let mut actions = vec![false; self.agents.len()];
        let mut proba_distb = vec![0.0; self.agents.len()]; // proba_distb must sum to 1
        for (agent1, idxs1) in unique_agents.iter(){
            for (agent2, idxs2) in unique_agents.iter(){
                if agent1 == agent2 {
                    continue;
                }

                let action1 = agent1.map_history_to_action();
                let action2 = agent2.map_history_to_action();
                let payoff = self.payoff_map.get(&(action1, action2)).unwrap();

                //
                for idx in idxs1.iter(){
                    payoffs[*idx as usize] = payoff.0;
                }
                for idx in idxs2.iter(){
                    payoffs[*idx as usize] = payoff.1;
                }


                //agent1.payoff += payoff.0;
                //agent2.payoff += payoff.1;
            }
        }

    }

}

*/