mod simulation;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::simulation::{agent::Agent, game::Game, test::{run_test, mega_test}};
use std::time;

fn main() {

   let t0 = time::Instant::now();
   
   let agents: Vec<Agent> = (0..1000)
      .map(|_| Agent::random_init(2))
      .collect();

   let board = match simulation::types::GameBoard::new(
      "prisoners_dilemma".to_string(),
      2
   ) {
      Ok(board) => board,
      Err(e) => panic!("Error creating game board: {}", e),
   };

   let game = Game::new(
      board,
      false,
      false
   );

   let mut game = match game {
      Ok(game) => game,
      Err(e) => panic!("Error creating game: {}", e),
   }; 
   game.run(2, agents);
   println!("took {:?}", t0.elapsed());
   game.dump_round_state_to_json("roundstates.json".to_string());
   let roundstates = simulation::utils::read_json("roundstates.json").unwrap();
   println!("Read {:?} roundstates from file", roundstates);
}
