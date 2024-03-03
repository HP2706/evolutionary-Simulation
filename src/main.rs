mod simulation;
use crate::simulation::agent::Agent;
use crate::simulation::types::{Mutation, PointMutation, GeneDuplication, SplitMutation};
use crate::simulation::game::{Game, RoundState};
use std::{collections::HashMap, hash::Hash};
use std::iter::{zip};
fn main() {
   //time
   let mut agents: Vec<Agent> = Vec::new();
   for _ in 0..20 {
       agents.push(Agent::random_init(2));
   }
   let mut game = Game::new(agents, false);
   game.run(1);
   let roundstate = game.state.last().unwrap();
}
