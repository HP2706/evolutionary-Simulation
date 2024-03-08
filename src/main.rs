mod simulation;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use crate::simulation::{agent::Agent, game::Game, test::{run_test, mega_test}};

fn main() {

   mega_test(3);

   /* let agents = (0..10).
      into_par_iter().
      map(|i| Agent::random_init(2)).
      collect();

   let board = match simulation::types::GameBoard::new(
      "prisoners_dilemma".to_string(),
      3
   ) {
      Ok(board) => board,
      Err(e) => panic!("Error creating game board: {}", e),
   };

   let game = Game::new(
      board,
      agents,
      false
   );

   let mut game = match game {
      Ok(game) => game,
      Err(e) => panic!("Error creating game: {}", e),
   }; 
   game.run(1); */
  
}
