mod simulation;
use polars::frame::DataFrame;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::simulation::{agent::Agent, game::Game, test::{mega_test, run_test}, types::GameBoard};
use std::{borrow::BorrowMut, time};

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

   let mut df = match game.round_state_to_dataframe() {
      Ok(df) => df,
      Err(e) => panic!("Error creating dataframe: {}", e),
   };

   match game.dump_to_parquet(&mut df, "test.parquet".to_string()) {
      Ok(_) => (),
      Err(e) => panic!("Error dumping to parquet: {}", e),
   }

}
