mod simulation;
use crate::simulation::agent::Agent;
use crate::simulation::types::{Mutation, PointMutation, GeneDuplication, SplitMutation};
use crate::simulation::game::{Game, RoundState};
use std::{collections::HashMap, hash::Hash};
use std::iter::{zip};
fn main() {
   //time
   let t0 = std::time::Instant::now();
   let mut agents: Vec<Agent> = Vec::new();


   for _ in 0..1000 {
      agents.push(Agent::random_init(2));
   }
   let mut game = Game::new(agents, false);
   match game.run(1000) {
      Ok(_) => (),
      Err(e) => println!("Error in running game: {:?}", e)
   }
   //let roundstate = game.state.last().unwrap();
   println!("Time taken: {:?}", t0.elapsed());
}
