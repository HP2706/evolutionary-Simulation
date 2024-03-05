mod simulation;
use crate::simulation::agent::Agent;
use crate::simulation::{game::{GamePlay, RoundState}};
use std::{collections::HashMap, hash::Hash};
use std::iter::{zip};
use std::option::Option;
use rand::Rng;
use rayon::prelude::*;

fn main() {
   //time
   let t0 = std::time::Instant::now();
   let mut agents: Vec<Agent> = Vec::new();

   println!("took time sequantial: {:?}", t0.elapsed());

   let t0 = std::time::Instant::now();
   let agents: Vec<Agent> = (0..100000)
        .into_par_iter() // Use into_par_iter for parallel iteration
        .map(|_| Agent::random_init(2)) // Initialize each agent in parallel
        .collect(); //
   
   let mut game = GamePlay::new(agents, false, None);
   match game.run(10) {
      Ok(_) => (),
      Err(e) => println!("Error in running game: {:?}", e)
   }

   game.dump_history_to_file("game_history.json");
   //let roundstate = game.state.last().unwrap();
   println!("Time taken: {:?}", t0.elapsed());
   println!("Game state length: {:?}", game.gamestate.len());

}
