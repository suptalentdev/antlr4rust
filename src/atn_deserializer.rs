use crate::atn_deserialization_options::ATNDeserializationOptions;
use crate::atn::ATN;
use crate::interval_set::IntervalSet;
use crate::atn_state::ATNState;
use crate::transition::Transition;
use uuid::Uuid;
use crate::atn_type::ATNType;

use std::str::FromStr;

use std::str::Chars;
use byteorder::WriteBytesExt;
use byteorder::LittleEndian;
use crate::atn_state::BaseATNState;
use crate::atn_state::ATNStateType;
use crate::atn_state::ATNDecisionState;
use crate::atn_state::ATNBlockStart;
use crate::atn_state::*;
use crate::transition::*;
use std::any::Any;
use std::mem;
use std::io::{stdout, Write};
use crate::lexer_action::LexerAction;
use crate::int_stream::EOF;

lazy_static! {
    static ref BASE_SERIALIZED_UUID: Uuid =
        Uuid::from_str("33761B2D-78BB-4A43-8B0B-4F5BEE8AACF3").unwrap();
    static ref ADDED_PRECEDENCE_TRANSITIONS: Uuid =
        Uuid::from_str("1DA0C57D-6C06-438A-9B27-10BCB3CE0F61").unwrap();
    static ref ADDED_LEXER_ACTIONS: Uuid =
        Uuid::from_str("AADB8D7E-AEEF-4415-AD2B-8204D6CF042E").unwrap();
    static ref ADDED_UNICODE_SMP: Uuid =
        Uuid::from_str("59627784-3BE5-417A-B9EB-8131A7286089").unwrap();
    static ref SUPPORTED_UUIDS: Vec<Uuid> = vec![
        *BASE_SERIALIZED_UUID,
        *ADDED_PRECEDENCE_TRANSITIONS,
        *ADDED_LEXER_ACTIONS,
        *ADDED_UNICODE_SMP,
    ];
}

const SERIALIZED_VERSION: isize = 3;

pub struct ATNDeserializer {
    deserialization_options: ATNDeserializationOptions,
    data: Vec<u8>,
    pos: isize,
    uuid: String,
    //    pd:PhantomData<*const T>
}

pub unsafe fn cast<T: Transition>(tr: &dyn Transition) -> &T {
    let to = mem::transmute::<&dyn Transition, std::raw::TraitObject>(tr).data;
    mem::transmute::<*mut (), &T>(to)
}

impl ATNDeserializer {
    pub fn new(options: Option<ATNDeserializationOptions>) -> ATNDeserializer {
        ATNDeserializer {
            deserialization_options: options.unwrap_or(ATNDeserializationOptions::default()),
            data: Vec::new(),
            pos: 0,
            uuid: String::new(),
            //            pd: PhantomData,
        }
    }

    fn String_in_slice(_a: String, _list: Vec<String>) -> isize {
        unimplemented!()
    }

    fn is_feature_supported(&self, _feature: String, _actualUUID: String) -> bool {
        unimplemented!()
    }

    pub fn deserialize(&self, data: Chars) -> ATN {
        let mut data = data.clone().map(|ch| ch as isize - 2);

        self.check_version(data.next().unwrap() + 2);

        let _uuid = self.check_uuid(&mut data);

        let mut atn = self.read_atn(&mut data);

        self.read_states(&mut atn, &mut data);
        self.read_rules(&mut atn, &mut data);
        self.read_modes(&mut atn, &mut data);

        let mut sets = self.read_sets(&mut atn, &mut data, |data| data.next().unwrap());

        sets.extend(self.read_sets(&mut atn, &mut data,
                                   |data|
                                       (data.next().unwrap() & 0xFFFF) | data.next().unwrap() << 16));

        self.read_edges(&mut atn, &mut data, &sets);
        self.read_decisions(&mut atn, &mut data);
        if atn.grammar_type == ATNType::LEXER {
            self.read_lexer_actions(&mut atn, &mut data);
        }
        self.mark_precedence_decisions(&mut atn, &mut data);
        if self.deserialization_options.is_verify() {
            self.verify_atn(&mut atn, &mut data);
        }
        //      TODO parser
        //        if a.deserializationOptions.generateRuleBypassTransitions && atn.grammarType == ATNTypeParser {
        //            a.generateRuleBypassTransitions(atn)
        //            a.verifyATN(atn)
        //        }

        atn
    }

    fn reset(&self, _data: Vec<u8>) {
        unimplemented!()
    }

    fn check_version(&self, version: isize) {
        if version != self::SERIALIZED_VERSION {
            panic!(
                "Could not deserialize ATN with version {} (expected {})",
                version, SERIALIZED_VERSION
            );
        }
    }

    fn check_uuid(&self, data: &mut Iterator<Item=isize>) -> Uuid {
        //rust uses UTF-8 encoding so we need explicitly convert unicode
        //codepoint numbers to bytes
        let mut bytes = Vec::new();
        for i in data.take(8) {
            bytes.write_u16::<LittleEndian>(i as u16);
        }

        bytes.reverse();
        let uuid = Uuid::from_bytes(bytes.as_slice()).unwrap();
        if !SUPPORTED_UUIDS.contains(&uuid) {
            panic!("Could not deserialize ATN with UUID {}", uuid)
        }
        uuid
    }

    fn read_atn(&self, data: &mut Iterator<Item=isize>) -> ATN {
        let atn = ATN::new_atn(
            match data.next() {
                Some(0) => ATNType::LEXER,
                Some(1) => ATNType::PARSER,
                _ => panic!("invalid ATN type"),
            },
            data.next().unwrap(),
        );

        atn
    }

    fn read_states(&self, atn: &mut ATN, data: &mut dyn Iterator<Item=isize>) {
        //        let loop_back_states = Vec::<(BaseATNState,isize)>::new();
        //        let end_states = Vec::<(BaseATNState,isize)>::new();
        let states_count = data.next().unwrap() as usize;
        for i in 0..states_count {
            let state_type = data.next().unwrap();
            if state_type == ATNSTATE_INVALID_STATE_NUMBER {
                atn.add_state(self.state_factory(ATNSTATE_INVALID_TYPE, -1, i));
                panic!("why invalid state serialized?");
                continue;
            }

            let mut rule_index = data.next().unwrap();
            if rule_index == 0xFFFF {
                rule_index = -1;
            }
            let state = self.state_factory(state_type, rule_index, i);

            match state.get_state_type() {
                ATNStateType::DecisionState {
                    state: ATNDecisionState::BlockStartState { mut end_state, .. },
                    ..
                } => end_state = data.next().unwrap() as ATNStateRef,
                ATNStateType::LoopEndState(mut loop_back) => {
                    loop_back = data.next().unwrap() as ATNStateRef
                }
                _ => (),
            }
            atn.add_state(state);
        }

        let num_non_greedy = data.next().unwrap();
        println!("num_non_greedy {}", num_non_greedy);
        for _ in 0..num_non_greedy {
            let st = data.next().unwrap() as usize;
            if let ATNStateType::DecisionState {
                nongreedy: mut ng, ..
            } = atn.states.get(st).unwrap().get_state_type()
            {
                ng = true
            }
        }

        //if (supportsPrecedencePredicates)
        if true {
            let num_precedence_states = data.next().unwrap();
            for _ in 0..num_precedence_states {
                let st = data.next().unwrap() as usize;
                if let ATNStateType::RuleStartState {
                    is_left_recursive: mut left_rec,
                    ..
                } = atn.states.get(st).unwrap().get_state_type()
                {
                    left_rec = true
                }
            }
        }
    }

    fn read_rules(&self, atn: &mut ATN, data: &mut Iterator<Item=isize>) {
        let nrules = data.next().unwrap() as usize;
//        if atn.grammar_type == ATNType::LEXER {
//            atn.rule_to_token_type.resize(nrules, 0)
//        }

        atn.rule_to_start_state.resize(nrules, 0);
        for i in 0..nrules {
            let s = data.next().unwrap() as usize;
            atn.rule_to_start_state[i] = s;
            if atn.grammar_type == ATNType::LEXER {
                let token_type = data.next().unwrap();

                atn.rule_to_token_type.push(token_type);
            }
        }
        println!("rule_to_token_type {:?}", atn.rule_to_token_type);
        println!("rule_to_start_state {:?}", atn.rule_to_start_state);

        atn.rule_to_stop_state.resize(nrules, 0);
        for i in 0..atn.states.len() {
            let state = atn.states.get(i).unwrap();
            if let ATNStateType::RuleStopState = state.get_state_type() {
                let rule_index = state.get_rule_index();
                atn.rule_to_stop_state[rule_index] = i;
                let start_state = atn.states
                    .get_mut(atn.rule_to_start_state[rule_index])
                    .unwrap();
                if let ATNStateType::RuleStartState {
                    stop_state: mut stop,
                    ..
                } = start_state.get_state_type()
                {
                    stop = i
                }
            }
        }
    }

    fn read_modes(&self, atn: &mut ATN, data: &mut dyn Iterator<Item=isize>) {
        let nmodes = data.next().unwrap();
        for _i in 0..nmodes {
            atn.mode_to_start_state.push(data.next().unwrap() as usize);
        }
    }

    fn read_sets<T: Iterator<Item=isize>>(
        &self,
        _atn: &mut ATN,
        data: &mut T,
        readUnicode: fn(&mut T) -> isize,
    ) -> Vec<IntervalSet> {
        let nsets = data.next().unwrap();
        let mut sets = Vec::new();
        for i in 0..nsets {
            let intervals = data.next().unwrap();

            let mut set = IntervalSet::new_interval_set();

            if data.next().unwrap() != 0 {
                set.add_one(-1)
            }

            for i in 0..intervals {
                set.add_range(readUnicode(data), readUnicode(data));
            }

            sets.push(set);
        }

        sets
    }

    fn read_edges(
        &self,
        atn: &mut ATN,
        data: &mut Iterator<Item=isize>,
        sets: &Vec<IntervalSet>,
    ) {
        let nedges = data.next().unwrap();
        println!("transitions {}", nedges);
        for _i in 0..nedges {
            let src = data.next().unwrap() as usize;
            let trg = data.next().unwrap() as usize;
            let ttype = data.next().unwrap();
            let arg1 = data.next().unwrap();
            let arg2 = data.next().unwrap();
            let arg3 = data.next().unwrap();

            let transition = self.edge_factory(atn, ttype, src, trg, arg1, arg2, arg3, sets);
            println!("created transition from {} {:?}", src, transition);
            atn.states.get_mut(src).unwrap().add_transition(transition);
        }


        for atn_state in &atn.states {
            for i in atn_state.get_transitions() {
                match i.get_serialization_type() {
                    TransitionType::TRANSITION_RULE => {
                        println!("TRANSITION_RULE");
                        let tr = i.as_ref();
                        let tr = unsafe { cast::<RuleTransition>(tr) };
                        let target = atn.states.get(tr.get_target()).unwrap();

                        let outermost_prec_return =
                            if let ATNStateType::RuleStartState { is_left_recursive: true, .. } = atn.states
                                .get(atn.rule_to_start_state[target.get_rule_index()])
                                .unwrap()
                                .get_state_type()
                            {
                                if tr.precedence == 0 {
                                    target.get_rule_index() as isize
                                } else {
                                    -1
                                }
                            } else {
                                -1
                            };

                        let return_tr = EpsilonTransition {
                            target: tr.follow_state,
                            outermost_precedence_return: outermost_prec_return,
                        };
                        //                        atn.states
                        //                            .get_mut(atn.rule_to_stop_state[target.get_rule_index()])
                        //                            .unwrap()
                        //                            .add_transition(Box::new(return_tr));
                    }
                    _ => continue,
                }
            }
        }

        for i in 0..atn.states.len() {
            let atn_state = atn.states.get(i).unwrap();
            match atn_state.get_state_type() {
                ATNStateType::DecisionState {
                    state: ATNDecisionState::BlockStartState { end_state, en },
                    ..
                } => {

                    //                    if *end_state == 0 { panic!("invalid state")}

                    // todo missed part
                }
                //                ATNStateType::DecisionState {state:ATNDecisionState::PlusLoopBack,..} =>{
                //                    for tr in atn_state.get_transitions(){
                //                        if let ATNStateType::DecisionState {
                //                                        state:ATNDecisionState::BlockStartState {
                //                                            en:ATNBlockStart::PlusBlockStart(loopBack),..},..}
                //                        = atn.states.get_mut(tr.get_target()).unwrap().get_state_type_mut(){
                //                            *loopBack = i;
                //
                //                        }
                //                    }
                //                }
                x => { /*println!("{:?}",x);*/ }
            }
        }
    }

    fn read_decisions(&self, atn: &mut ATN, _data: &mut Iterator<Item=isize>) {
        let ndecisions = _data.next().unwrap();
        for i in 0..ndecisions {
            let s = _data.next().unwrap() as usize;
            let dec_state: &mut Box<ATNState> = atn.states.get_mut(s).unwrap();
            atn.decision_to_state.push(s);
            if let ATNStateType::DecisionState { decision, .. } = dec_state.get_state_type_mut() {
                *decision = i
            }
        }
    }

    fn read_lexer_actions(&self, atn: &mut ATN, _data: &mut Iterator<Item=isize>) {
        //lexer actions are always supported here
        let nactions = _data.next().unwrap() as usize;

        for i in 0..nactions {
            let action_type = _data.next().unwrap();

            let mut data1 = _data.next().unwrap();
            if data1 == 0xFFFF {
                data1 = -1;
            }
            let mut data2 = _data.next().unwrap();
            if data2 == 0xFFFF {
                data2 = -1;
            }

            let lexer_action = self.lexer_action_factory(action_type, data1, data2);

            atn.lexer_actions.push(lexer_action);
        }
    }

    fn generate_rule_bypass_transitions(&self, _atn: &mut ATN, _data: &mut Iterator<Item=isize>) {
        unimplemented!()
    }

    fn generate_rule_bypass_transition(
        &self,
        _atn: &mut ATN,
        _data: &mut Iterator<Item=isize>,
        _idx: isize,
    ) {
        unimplemented!()
    }

    fn state_is_end_state_for(&self, _state: &ATNState, _idx: isize) -> Box<ATNState> {
        unimplemented!()
    }

    fn mark_precedence_decisions(&self, atn: &mut ATN, data: &mut Iterator<Item=isize>) {
        //TODO
    }

    fn verify_atn(&self, _atn: &mut ATN, _data: &mut Iterator<Item=isize>) {
        //TODO
    }

    fn check_condition(&self, _condition: bool, _message: String) {
        unimplemented!()
    }

    fn read_int(&self) -> isize {
        unimplemented!()
    }

    fn read_int32(&self) -> isize {
        unimplemented!()
    }

    fn create_byte_to_hex() -> Vec<String> {
        unimplemented!()
    }

    fn read_uuid(&self) -> String {
        unimplemented!()
    }

    fn edge_factory(
        &self,
        _atn: &ATN,
        type_index: isize,
        _src: ATNStateRef,
        target: ATNStateRef,
        arg1: isize,
        arg2: isize,
        arg3: isize,
        sets: &Vec<IntervalSet>,
    ) -> Box<Transition> {
//        //        let target = atn.states.get
//        let mut base = BaseTransition {
//            target: trg,
//            //            is_epsilon: false,
//            //            label: 0,
//            interval_set: IntervalSet::new_interval_set(),
//        };

        match type_index {
            TRANSITION_EPSILON => Box::new(EpsilonTransition {
                target,
                outermost_precedence_return: 0,
            }),
            TRANSITION_RANGE => Box::new(RangeTransition {
                target,
                start: if arg3 != 0 {
                    super::token::TOKEN_EOF
                } else {
                    arg1
                },
                stop: arg2,
            }),
            TRANSITION_RULE => {
//                base.set_target(arg1 as usize);
                Box::new(RuleTransition {
                    target: arg1 as usize,
                    follow_state: target,
                    rule_index: arg2,
                    precedence: arg3,
                })
            }
            TRANSITION_PREDICATE => Box::new(PredicateTransition {
                target,
                is_ctx_dependent: arg3 != 0,
                rule_index: arg1,
                predIndex: arg2,
            }),
            TRANSITION_ATOM => Box::new(AtomTransition {
                target,
                label: if arg3 != 0 { EOF } else { arg1 },
            }),
            TRANSITION_ACTION => Box::new(ActionTransition {
                target,
                is_ctx_dependent: arg3 != 0,
                rule_index: arg1,
                action_index: arg2,
                pred_index: 0,
            }),
            TRANSITION_SET => {
                Box::new(SetTransition {
                    target,
                    set: sets[arg1 as usize].clone(),
                })
            }
            TRANSITION_NOTSET => {
                Box::new(NotSetTransition {
                    target,
                    set: sets[arg1 as usize].clone(),
                })
            }
            TRANSITION_WILDCARD => Box::new(WildcardTransition { target }),
            TRANSITION_PRECEDENCE => Box::new(PrecedencePredicateTransition {
                target,
                precedence: arg1,
            }),
            _ => panic!("invalid transition type"),
        }
    }

    fn state_factory(
        &self,
        typeIndex: isize,
        ruleIndex: isize,
        state_number: usize,
    ) -> Box<ATNState> {
        let mut state = BaseATNState::new_base_atnstate();
        state.state_number = state_number;
        state.rule_index = ruleIndex as usize;
        state.state_type_id = typeIndex;
        state.state_type = match typeIndex {
            ATNSTATE_INVALID_TYPE => ATNStateType::InvalidState,
            ATNSTATE_BASIC => ATNStateType::BasicState,
            ATNSTATE_RULE_START => ATNStateType::RuleStartState {
                stop_state: 0,
                is_left_recursive: false,
            },
            ATNSTATE_BLOCK_START => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: 0,
                    en: ATNBlockStart::BasicBlockStart,
                },
            },
            ATNSTATE_PLUS_BLOCK_START => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: 0,
                    en: ATNBlockStart::PlusBlockStart(0),
                },
            },
            ATNSTATE_STAR_BLOCK_START => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: 0,
                    en: ATNBlockStart::StarBlockStart,
                },
            },
            ATNSTATE_TOKEN_START => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::TokenStartState,
            },
            ATNSTATE_RULE_STOP => ATNStateType::RuleStopState,
            ATNSTATE_BLOCK_END => ATNStateType::BlockEndState(0),
            ATNSTATE_STAR_LOOP_BACK => ATNStateType::StarLoopbackState,
            ATNSTATE_STAR_LOOP_ENTRY => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::StarLoopEntry {
                    loop_back_state: 0,
                    is_precedence: false,
                },
            },
            ATNSTATE_PLUS_LOOP_BACK => ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::PlusLoopBack,
            },
            ATNSTATE_LOOP_END => ATNStateType::LoopEndState(0),

            _ => panic!("invalid ATN state type"),
        };
        println!("created state {} {:?}", state_number, state.state_type);
        Box::new(state)
    }

    fn lexer_action_factory(
        &self,
        typeIndex: isize,
        data1: isize,
        data2: isize,
    ) -> Box<LexerAction> {
        unimplemented!()
    }
}
