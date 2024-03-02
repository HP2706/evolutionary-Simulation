use std::collections::HashMap;
use crate::simulation::types::{Mutation, PointMutation, GeneDuplication, SplitMutation};
use rand::Rng;

#[derive(Hash, Debug)]
pub struct Agent {
    // attributes: memory, strategy, fitness
    pub memory_len : usize, // m
    pub history: Vec<bool>, // length m [a_m-1, a_m-2, ...,a_1, a_0]
    //where a_0 is the opponent's last action, a_1 is the agent's last action, and so on
    pub genome: Vec<bool>, // the strategy length n = 2^m [b_n-1, b_n-2, ...,b_1, b_0] the genome
}

impl Clone for Agent {
    fn clone(&self) -> Agent {
        Agent {
            memory_len: self.memory_len,
            history: self.history.clone(),
            genome: self.genome.clone(),
        }
    }
}

impl PartialEq for Agent {
    fn eq(&self, other: &Self) -> bool {
        self.genome == other.genome
    }
}

impl Eq for Agent {}

impl Agent {
    pub fn random_init(memory_len : u32) -> Agent {
        let history = (0..memory_len).map(|_| rand::random::<bool>()).collect();
        let genome = (0..2usize.pow(memory_len)).map(|_| rand::random::<bool>()).collect();
        Agent {
            memory_len : memory_len as usize,
            history: history,
            genome: genome,
        }

    }

    pub fn new(genome : Vec<bool>, history : Vec<bool>, memory_len : u32) -> Agent {
        if genome.len() != 2usize.pow(history.len() as u32) {
            panic!(
                "Strategy and memory length mismatch got genome len {} and history len {} 
                expected genome len of 2^history len {}", 
                genome.len(), history.len(), 2usize.pow(history.len() as u32)
            );
        }

        Agent {
            memory_len : memory_len as usize,
            history: history,
            genome: genome,
        }
    }

    pub fn map_history_to_action(&self) -> bool {
        //maps a binary sequence to an integer
        let idx = self.history.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u32));
        return self.genome[idx as usize];
    }   

    pub fn add_memory(&mut self, action : [bool; 2]) {
        //we add the action pair from the last round to the memory    
        // we pop the oldest action pair and push the new one 
        if self.memory_len > self.history.len() {
            self.history.extend(action.iter());
        } else {
            self.history.drain(0..2);
            self.history.extend(action.iter());
        }
    }

    pub fn get_action(&mut self) -> bool {
        //given current history, return the action
        let action = self.genome[self.map_history_to_action() as usize];
        return action;
    }

    pub fn mutate(&mut self, mutation: Mutation) {
        match mutation {
            Mutation::PointMutation(PointMutation) => {
                //change randon symbol in the strategy
                let index = rand::random::<usize>() % self.genome.len();
                self.genome[index] = !self.genome[index]; //flip the action
            }
            Mutation::GeneDuplication(GeneDuplication) => {
               //attach a copy to the genome/S to itself
                self.genome.extend(self.genome.clone());
            }
            Mutation::SplitMutation(SplitMutation) => {
                //remove first half of the S 
                let half = self.genome.len() / 2;
                self.genome = self.genome.split_off(half);
            }
        }
    }

}
