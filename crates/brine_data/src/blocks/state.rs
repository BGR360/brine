//! TODO: more information about how state id translates to combinations of states.

use std::{collections::HashMap, fmt};

use minecraft_data_rs::models::block::{
    Block as McBlock, State as McState, StateType as McStateType,
};

use super::block::IndexType;

pub type BlockState<'a> = HashMap<&'a str, StateValue<'a>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateValue<'a> {
    Enum(&'a str),
    Bool(bool),
    Int(i32),
}

impl<'a> StateValue<'a> {
    #[inline]
    pub fn as_enum_value(&self) -> Option<&'_ str> {
        if let Self::Enum(name) = self {
            Some(name)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i32> {
        if let Self::Int(i) = self {
            Some(*i)
        } else {
            None
        }
    }
}

impl<'a> fmt::Debug for StateValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enum(value) => write!(f, "{}", value),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Int(i) => write!(f, "{}", i),
        }
    }
}

impl<'a> fmt::Display for StateValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl<'a> From<&'a str> for StateValue<'a> {
    fn from(enum_value: &'a str) -> Self {
        Self::Enum(enum_value)
    }
}

impl<'a> From<bool> for StateValue<'a> {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl<'a> From<i32> for StateValue<'a> {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

pub(crate) trait McStateExt {
    fn possible_values(&self) -> Vec<StateValue<'_>>;
}

impl McStateExt for McState {
    fn possible_values(&self) -> Vec<StateValue<'_>> {
        match self.state_type {
            McStateType::Enum => self
                .values
                .as_ref()
                .unwrap()
                .iter()
                .map(|value| StateValue::Enum(value.as_str()))
                .collect(),

            // See net.minecraft.world.level.block.state.properties.BooleanProperty
            McStateType::Bool => vec![StateValue::Bool(true), StateValue::Bool(false)],

            // See net.minecraft.world.level.block.state.properties.IntegerProperty
            //
            // XXX: the Minecraft source code inserts integer values into a
            // HashSet before registering them, does that mean I have no way of
            // knowing the order of assignment??? Idk, just assuming they're
            // assigned in order for now...
            McStateType::Int => (0..self.num_values)
                .map(|i| StateValue::Int(i as i32))
                .collect(),
        }
    }
}

pub(crate) trait McBlockExt {
    fn possible_block_states(&self) -> PossibleBlockStates<'_>;
}

impl McBlockExt for McBlock {
    fn possible_block_states(&self) -> PossibleBlockStates<'_> {
        let state_values = self
            .states
            .as_ref()
            .map(|states| {
                states
                    .iter()
                    .map(|state| (state.name.as_str(), state.possible_values()))
                    .collect()
            })
            .unwrap_or_default();

        PossibleBlockStates { state_values }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PossibleBlockStates<'a> {
    state_values: Vec<(&'a str, Vec<StateValue<'a>>)>,
}

impl<'a> PossibleBlockStates<'a> {
    pub fn get_nth(&self, mut n: IndexType) -> BlockState<'a> {
        let mut state = HashMap::default();

        for (state_name, state_values) in self.state_values.iter().rev() {
            let num_values = state_values.len() as IndexType;
            let state_index = n % num_values;
            let remainder = n / num_values;

            let value = state_values[state_index as usize];
            state.insert(*state_name, value);

            n = remainder;
        }

        state
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use maplit::hashmap;

    fn test_bool_state() -> McState {
        McState {
            name: String::from("test_bool"),
            state_type: McStateType::Bool,
            num_values: 2,
            ..Default::default()
        }
    }

    fn test_int_state() -> McState {
        McState {
            name: String::from("test_int"),
            state_type: McStateType::Int,
            num_values: 3,
            ..Default::default()
        }
    }

    fn test_enum_state() -> McState {
        McState {
            name: String::from("test_enum"),
            state_type: McStateType::Enum,
            values: Some(vec![
                String::from("a"),
                String::from("b"),
                String::from("c"),
            ]),
            num_values: 3,
        }
    }

    mod state_values {
        use super::*;

        #[test]
        fn no_states() {
            let block = McBlock::default();

            let states = block.possible_block_states();

            let expected = PossibleBlockStates::default();

            assert_eq!(states, expected);
        }

        #[test]
        fn bool_state() {
            let block = McBlock {
                states: Some(vec![test_bool_state()]),
                ..Default::default()
            };

            let states = block.possible_block_states();

            let expected = PossibleBlockStates {
                state_values: vec![(
                    "test_bool",
                    vec![StateValue::Bool(false), StateValue::Bool(true)],
                )],
            };

            assert_eq!(states, expected);
        }

        #[test]
        fn int_state() {
            let block = McBlock {
                states: Some(vec![test_int_state()]),
                ..Default::default()
            };

            let states = block.possible_block_states();

            let expected = PossibleBlockStates {
                state_values: vec![(
                    "test_int",
                    vec![StateValue::Int(0), StateValue::Int(1), StateValue::Int(2)],
                )],
            };

            assert_eq!(states, expected);
        }

        #[test]
        fn enum_state() {
            let block = McBlock {
                states: Some(vec![test_enum_state()]),
                ..Default::default()
            };

            let states = block.possible_block_states();

            let expected = PossibleBlockStates {
                state_values: vec![(
                    "test_enum",
                    vec![
                        StateValue::Enum("a"),
                        StateValue::Enum("b"),
                        StateValue::Enum("c"),
                    ],
                )],
            };

            assert_eq!(states, expected);
        }
    }

    mod block_states {
        use super::*;

        #[test]
        fn empty() {
            let possible_states = PossibleBlockStates {
                state_values: Default::default(),
            };

            assert_eq!(possible_states.get_nth(0), HashMap::default());
        }

        #[test]
        fn bool_state() {
            let block = McBlock {
                states: Some(vec![test_bool_state()]),
                ..Default::default()
            };
            let possible_states = block.possible_block_states();

            assert_eq!(
                possible_states.get_nth(0),
                hashmap! {
                    "test_bool" => StateValue::Bool(true)
                }
            );
            assert_eq!(
                possible_states.get_nth(1),
                hashmap! {
                    "test_bool" => StateValue::Bool(false)
                }
            );
        }

        #[test]
        fn int_then_bool_state() {
            let block = McBlock {
                states: Some(vec![test_int_state(), test_bool_state()]),
                ..Default::default()
            };
            let possible_states = block.possible_block_states();

            assert_eq!(
                possible_states.get_nth(0),
                hashmap! {
                    "test_int" => StateValue::Int(0),
                    "test_bool" => StateValue::Bool(true)
                }
            );
            assert_eq!(
                possible_states.get_nth(1),
                hashmap! {
                    "test_int" => StateValue::Int(0),
                    "test_bool" => StateValue::Bool(false)
                }
            );
            assert_eq!(
                possible_states.get_nth(2),
                hashmap! {
                    "test_int" => StateValue::Int(1),
                    "test_bool" => StateValue::Bool(true)
                }
            );
            assert_eq!(
                possible_states.get_nth(3),
                hashmap! {
                    "test_int" => StateValue::Int(1),
                    "test_bool" => StateValue::Bool(false)
                }
            );
            assert_eq!(
                possible_states.get_nth(4),
                hashmap! {
                    "test_int" => StateValue::Int(2),
                    "test_bool" => StateValue::Bool(true)
                }
            );
            assert_eq!(
                possible_states.get_nth(5),
                hashmap! {
                    "test_int" => StateValue::Int(2),
                    "test_bool" => StateValue::Bool(false)
                }
            );
        }
    }
}
