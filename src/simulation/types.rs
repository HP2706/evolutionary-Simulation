use serde::{Serialize, Deserialize};
use serde::ser::{SerializeMap, Serializer, SerializeStruct};
use serde::de::{self, Deserializer, Visitor, MapAccess};
use std::fmt;
use crate::simulation::agent::Agent;
use std::collections::HashMap;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameBoard {
    payoff_matrix: HashMap<Vec<bool>, Vec<f64>>,
    pub n_players: u32,
}

impl GameBoard {
    pub fn new(game_name: String, n_players : u32) -> Result<GameBoard, String> {
        
        let mut payoff_matrix : HashMap<Vec<bool>, Vec<f64>> = HashMap::new();
        match game_name.as_str() {
            "prisoners_dilemma" => {
                match n_players  {
                2 => {
                    payoff_matrix.insert(vec![true, true], vec![5.0, 10.0]); // agent 1 cooperates, agent 2 cooperates
                    payoff_matrix.insert(vec![true, false], vec![1.0, 15.0]); // agent 1 cooperates, agent 2 defects
                    payoff_matrix.insert(vec![false, true], vec![10.0, 10.0]); // agent 1 defects, agent 2 cooperates
                    payoff_matrix.insert(vec![false, false], vec![0.0, 5.0]);  // agent 1 defects, agent 2 defects
                }, 
                3 => {
                    // All cooperate
                    payoff_matrix.insert(vec![true, true, true], vec![5.0, 5.0, 5.0]);
                    // Two cooperate, one defects
                    payoff_matrix.insert(vec![true, true, false], vec![3.0, 3.0, 10.0]);
                    payoff_matrix.insert(vec![true, false, true], vec![3.0, 10.0, 3.0]);
                    payoff_matrix.insert(vec![false, true, true], vec![10.0, 3.0, 3.0]);
                    // One cooperates, two defect
                    payoff_matrix.insert(vec![true, false, false], vec![1.0, 0.0, 0.0]);
                    payoff_matrix.insert(vec![false, true, false], vec![0.0, 1.0, 0.0]);
                    payoff_matrix.insert(vec![false, false, true], vec![0.0, 0.0, 1.0]);
                    // All defect
                    payoff_matrix.insert(vec![false, false, false], vec![0.0, 0.0, 0.0]);
                }

                _ => return Err(format!("Invalid number of players: {} not implemted yet", n_players)),
            };
            },
            _ => return Err(format!("Invalid game name: {} not implemted yet", game_name)),
        }
        
        Ok(GameBoard {
            payoff_matrix: payoff_matrix,
            n_players: n_players,
        })
    }

    pub fn get_payoff(&self, action: &Vec<bool>) -> Vec<f64> {

        if action.len() != self.n_players as usize {
            panic!("Invalid action length: {} expected {}", action.len(), self.n_players);
        }

        match self.payoff_matrix.get(action) {
            Some(payoff) => payoff.clone(),
            None => panic!("Invalid action: {:?}", action),
        }
    }


}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentRoundData {
    pub count : u32, 
    pub score : f64,
    pub fitness : f64,
    pub population_share : f64
}

impl AgentRoundData {
    pub fn from(count: u32, score : f64, fitness : f64, population_share : f64) -> AgentRoundData {
        AgentRoundData {
            count: count,
            score: score,
            fitness: fitness,
            population_share: population_share
        }
    }
}

///this holds features like count and population share which are useful to have precomputed for each round
/// it is used in the hashmap with agents IE HashMap<Agent, AgentMetaData>
/// # Variables:
///    pub count: u32 - the number of agents with the same genome
///     pub population_share: f64 - the share of the population with the same genome
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentMetaData {
    pub count : u32,
    pub population_share : f64,
}

impl AgentMetaData {
    pub fn new() -> AgentMetaData {
        AgentMetaData {
            count: 0,
            population_share: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SerializationAgentData {
    agent_data : Agent,
    round_data : AgentRoundData
}

impl SerializationAgentData {
    fn new(agent_data : Agent, round_data : AgentRoundData) -> SerializationAgentData {
        SerializationAgentData {
            agent_data: agent_data,
            round_data: round_data
        }
    }
}


/// This holds the state of the game at a given round
/// # Variables:
///     pub round_number: u32 - the round number
///     pub agent_data: HashMap<Agent, AgentRoundData> - the data for each agent in the round
#[derive(Debug, Clone)]
pub struct RoundState {
    pub round_number : u32,
    pub agent_data : HashMap<Agent, AgentRoundData>,
}

impl Serialize for RoundState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RoundState", 2)?;
        state.serialize_field("round_number", &self.round_number)?;
        
        // Serialize agent_data with agent.id as the key
        let agent_data_map = self.agent_data.iter().
            map(|(agent, data)| 
            (&agent.id, SerializationAgentData::new(agent.clone(), data.clone()
        )));
        state.serialize_field("agent_data", &agent_data_map.collect::<HashMap<_, _>>())?;
        
        state.end()
    }
}

impl<'de> Deserialize<'de> for RoundState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { RoundNumber, AgentData }

        struct RoundStateVisitor;

        impl<'de> Visitor<'de> for RoundStateVisitor {
            type Value = RoundState;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RoundState")
            }

            fn visit_map<V>(self, mut map: V) -> Result<RoundState, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut round_number = None;
                let mut agent_data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::RoundNumber => {
                            if round_number.is_some() {
                                return Err(de::Error::duplicate_field("round_number"));
                            }
                            round_number = Some(map.next_value()?);
                        },
                        Field::AgentData => {
                            if agent_data.is_some() {
                                return Err(de::Error::duplicate_field("agent_data"));
                            }
                            // Deserialize into a temporary structure that mirrors the JSON
                            let temp_agent_data: HashMap<String, SerializationAgentData> = map.next_value()?;
                            // Transform into the expected HashMap<Agent, AgentRoundData>
                            agent_data = Some(temp_agent_data.into_iter().map(|(id, data)| {
                                // Assuming Agent can be constructed from its ID
                                let agent: Agent = data.agent_data;
                                (agent, data.round_data)
                            }).collect());
                        },
                    }
                }
                let round_number = round_number.ok_or_else(|| de::Error::missing_field("round_number"))?;
                let agent_data: HashMap<Agent, AgentRoundData> = agent_data.ok_or_else(|| de::Error::missing_field("agent_data"))?;
                Ok(RoundState { round_number, agent_data })
            }
        }

        const FIELDS: &'static [&'static str] = &["round_number", "agent_data"];
        deserializer.deserialize_struct("RoundState", FIELDS, RoundStateVisitor)
    }
}

impl RoundState {
    pub fn new(round_number: u32) -> RoundState {
        RoundState {
            round_number: round_number,
            agent_data: HashMap::new(),
        }
    }
}