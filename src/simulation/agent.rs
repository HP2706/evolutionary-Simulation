use rand::Rng;
use serde::{Serialize, Deserialize};
use rand_distr::{Distribution, Poisson};
#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct Agent {
    // attributes: memory, strategy, fitness
    pub id : String,
    pub memory_len : usize, // m
    pub history: Vec<bool>, // length m [a_m-1, a_m-2, ...,a_1, a_0]
    pub history_len : usize, // we maintain the length of the history to avoid recomputation
    //where a_0 is the opponent's last action, a_1 is the agent's last action, and so on
    pub genome: Vec<bool>, // the strategy length n = 2^m [b_n-1, b_n-2, ...,b_1, b_0] the genome
}

impl Clone for Agent {
    fn clone(&self) -> Agent {
        Agent {
            id: self.id.clone(),
            memory_len: self.memory_len,
            history: self.history.clone(),
            history_len: self.history_len,
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
        let genome : Vec<bool> = (0..2usize.pow(memory_len)).map(|_| rand::random::<bool>()).collect();
        Agent {
            id : Agent::genome_to_id(genome.clone()),
            memory_len : memory_len as usize,
            history: history,
            history_len: memory_len as usize,
            genome: genome,
        }

    }

    pub fn genome_to_id(genome : Vec<bool>) -> String {
        let mut id = String::new();
        for (i, &bit) in genome.iter().enumerate() {
           if bit {
              id.push('1');
           } else {
              id.push('0');
           }
        }
        return id;
     }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(&self).unwrap()
    }

    pub fn new(genome : Vec<bool>, history : Vec<bool>, memory_len : u32) -> Agent {
        if genome.len() != 2usize.pow(history.len() as u32) {
            panic!(
                "Strategy and memory length mismatch got genome len {} and history len {} 
                expected genome len of 2^history len {}", 
                genome.len(), history.len(), 2usize.pow(history.len() as u32)
            );
        }

        let len = history.len();

        Agent {
            id : Agent::genome_to_id(genome.clone()),
            memory_len : memory_len as usize,
            history: history,
            history_len: len,
            genome: genome,
        }
    }

    pub fn map_history_to_action(&self) -> bool {
        //maps a binary sequence to an integer
        println!("History: {:?}", self.history);
        let idx = self.history.iter().fold(0, |acc, &bit| (acc << 1) | (bit as u32));
        println!("Mapped history to action: {}", idx);
        println!("Genome: {:?}", self.genome);
        println!("output: {}", self.genome[idx as usize]);
        return self.genome[idx as usize];
    }   

    pub fn add_memory(&mut self, old_actions : [bool; 2]) {
        //we add the action pair from the last round to the memory    
        // we pop the oldest action pair and push the new one 
        if self.memory_len > self.history_len {
            self.history.extend(old_actions.iter());
            self.history_len += 2;
        } else {
            self.history.drain(0..2);
            self.history.extend(old_actions.iter());
        }
    }

    pub fn get_action(&mut self) -> bool {
        //given current history, return the action
        let action = self.genome[self.map_history_to_action() as usize];
        return action;
    }

    pub fn mutate(&mut self,  p_p : f64, p_d : f64, p_s : f64) -> bool {

        let mut rng = rand::thread_rng();
        //sample 3 one for each mutation type
        let mut mutated = false;

        let poisson = Poisson::new(p_p as f64).unwrap();
        let PointMutation_outcome : bool = poisson.sample(&mut rng) > 0.0;
         
        if PointMutation_outcome {
            println!("Point Mutation");
            let index = rand::random::<usize>() % self.genome.len();
            self.genome[index] = !self.genome[index]; 
            mutated = true;
        }

        /* let poisson = Poisson::new(p_d as f64).unwrap();
        let GeneDuplication_outcome = poisson.sample(&mut rng) > 0.0;
        
        if GeneDuplication_outcome {
            //attach a copy to the genome/S to itself
            self.genome.extend(self.genome.clone());
        } */

        let poisson = Poisson::new(p_s as f64).unwrap();
        let SplitMutation_outcome = poisson.sample(&mut rng) > 0.0;

        if SplitMutation_outcome {
            println!("Split Mutation");
            let half = self.genome.len() / 2;
            let start = if rand::thread_rng().gen() { 0 } else { half };
            let selected_half = self.genome[start..start + half].to_vec();
            self.genome.extend_from_slice(&selected_half);
            mutated = true;
        }
        return mutated;
    }

}
