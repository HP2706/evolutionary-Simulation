mod simulation;
use crate::simulation::agent::Agent;
use crate::simulation::{game::{GamePlay, RoundState}};
use std::{collections::HashMap, hash::Hash};
use std::iter::{zip};
use std::option::Option;
use rand::Rng;
fn main() {
   //time
   let t0 = std::time::Instant::now();
   let mut agents: Vec<Agent> = Vec::new();

   for _ in 0..100 {
      agents.push(Agent::random_init(2));
   }

   let mut game = GamePlay::new(agents, false, None);
   match game.run(10) {
      Ok(_) => (),
      Err(e) => println!("Error in running game: {:?}", e)
   }
   //let roundstate = game.state.last().unwrap();
   println!("Time taken: {:?}", t0.elapsed());
   //println!("Game state length: {:?}", game.gamestate.to_json());

}
