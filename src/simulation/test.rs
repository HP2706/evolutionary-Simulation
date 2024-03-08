use crate::simulation::{types::{RoundState, AgentRoundData, GameBoard}, agent::Agent, game::Game};
use std::collections::HashMap;
use std::collections;
use itertools::Itertools;
use approx::assert_abs_diff_eq;

#[derive(Clone, Debug)]
struct TestData {
    pub score : f64,
    pub fitness : f64
}

pub fn mega_test(n_players : u32){
    for _ in 0..100 {
        run_test(n_players);
    }
}

pub fn run_test(n_players : u32) {
    let game_board = match GameBoard::new("prisoners_dilemma".to_string(), n_players) {
        Ok(board) => board,
        Err(e) => panic!("Error creating game board: {}", e),
    }; 

    let agent1 = Agent::random_init(2);
    let agent2 = Agent::random_init(2);
    let agent3 = Agent::random_init( 2);
    let agent4 = Agent::random_init( 2);


    let mut agents = vec![agent1, agent2, agent3];
    let mut cloned_agents_map = Game::agents_to_hashmap(&agents);

    let mut cloned_agents = agents.clone();
    assert_eq!(agents.sort(), cloned_agents.sort());

    let mut game = match Game::new(
        game_board,
        false,
        true,
    ) {
        Ok(game) => game,
        Err(e) => panic!("Error creating game: {}", e),
    };


    let mut sorted_agents = cloned_agents.iter().collect::<Vec<_>>();
    sorted_agents.sort();
    let actions = sorted_agents.iter().sorted()
        .map(
            |agent| agent.get_action()
        )
        .collect::<Vec<bool>>();

    let scores = game.game_board.get_payoff(
        &actions
    );

    let mut weighted_scores : Vec<f64> = Vec::new();
    for (i, agent) in cloned_agents.iter().enumerate() {
        let score : Vec<f64> = scores.iter().zip(cloned_agents.iter()).enumerate().map(
            |(j , (_, agent))|
            {
                if i != j {
                    let agent_metadata = cloned_agents_map.get(agent).unwrap();
                    scores[i] * agent_metadata.population_share
                } else {0.0}
            }   
        ).collect();
        weighted_scores.push(score.iter().sum::<f64>());
    }

    let average_payoff = (weighted_scores.iter().zip(cloned_agents.iter())
        .map(
            |(weighted_score, agent)| 
            weighted_score * cloned_agents_map.get(agent).unwrap().population_share as f64
        ).sum::<f64>()) / n_players as f64; 
    // the average is simply the sum of the alrady weighted scores
    // we "manually" compute the average.
    let mut known_outcomes : HashMap<Agent, TestData> = HashMap::new();
    weighted_scores.iter().zip(cloned_agents.iter()).for_each(|(score, agent)| {
        known_outcomes.insert(agent.clone(), TestData{score: *score, fitness: *score - average_payoff});
    });

    game.run(1, agents );
    let state = game.rounds.last().unwrap();

    state.agent_data.iter().for_each(|(agent, agent_data)| {
        let known_outcome = known_outcomes.get(&agent).unwrap();
        assert_abs_diff_eq!(agent_data.score, known_outcome.score);
        assert_abs_diff_eq!(agent_data.fitness, known_outcome.fitness);
    });
    



}

