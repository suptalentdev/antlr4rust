use std::collections::HashSet;
use std::ops::Deref;

use bit_set::BitSet;

use crate::atn::ATN;
use crate::atn_config::ATNConfig;
use crate::atn_deserializer::cast;
use crate::atn_state::{ATNState, ATNStateType};
use crate::interval_set::IntervalSet;
use crate::parser_rule_context::ParserRuleContext;
use crate::prediction_context::PredictionContext;
use crate::semantic_context::SemanticContext::Precedence;
use crate::token::{TOKEN_EOF, TOKEN_EPSILON, TOKEN_INVALID_TYPE, TOKEN_MIN_USER_TOKEN_TYPE};
use crate::transition::{RuleTransition, TransitionType};
use crate::transition::TransitionType::TRANSITION_NOTSET;

pub struct LL1Analyzer<'a> {
    atn: &'a ATN,
}

impl LL1Analyzer<'_> {
    pub fn new(atn: &ATN) -> LL1Analyzer { LL1Analyzer { atn } }

    fn get_decision_lookahead(&self, s: &dyn ATNState) -> &Vec<IntervalSet> { unimplemented!() }

    pub fn look(&self,
//                atn: &ATN,
                s: &dyn ATNState,
                stop_state: Option<&dyn ATNState>,
                ctx: Option<&dyn ParserRuleContext>,
    ) -> IntervalSet {
        let mut r = IntervalSet::new();
        let look_ctx = ctx.map(|x|
            PredictionContext::from_rule_context(self.atn, x)
        );
        let mut looks_busy: HashSet<ATNConfig> = HashSet::new();
        let mut called_rule_stack = BitSet::new();
        self.look_work(
            s,
            stop_state,
            look_ctx,
            &mut r,
            &mut looks_busy,
            &mut called_rule_stack,
            true,
            true,
        );
        r
    }


    fn look_work(&self,
//                 atn:&ATN,
                 s: &dyn ATNState,
                 stop_state: Option<&dyn ATNState>,
                 mut ctx: Option<PredictionContext>,
                 look: &mut IntervalSet,
                 look_busy: &mut HashSet<ATNConfig>,
                 called_rule_stack: &mut BitSet,
                 see_thru_preds: bool,
                 add_eof: bool,
    ) {
        let c = ATNConfig::new(s.get_state_number(), 0, ctx.clone());
        if !look_busy.insert(c) { return; }

        if Some(s.get_state_number()) == stop_state.map(|x| x.get_state_number()) {
            match ctx {
                None => {
                    look.add_one(TOKEN_EPSILON);
                    return;
                }
                Some(x) if x.is_empty() && add_eof => {
                    look.add_one(TOKEN_EOF);
                    return;
                }
                _ => {}
            }
        }

        if let ATNStateType::RuleStopState = s.get_state_type() {
            match ctx {
                None => {
                    look.add_one(TOKEN_EPSILON);
                    return;
                }
                Some(x) if x.is_empty() && add_eof => {
                    look.add_one(TOKEN_EOF);
                    return;
                }
                Some(mut ctx) if ctx != PredictionContext::new_empty() => {
                    let removed = called_rule_stack.contains(s.get_rule_index());
                    called_rule_stack.remove(s.get_rule_index());
                    for i in 0..ctx.length() {
                        self.look_work(
                            self.atn.states[ctx.get_return_state(i) as usize].as_ref(),
                            stop_state,
                            ctx.take_parent(i),
                            look,
                            look_busy,
                            called_rule_stack,
                            see_thru_preds,
                            add_eof,
                        )
                    }
                    if removed {
                        called_rule_stack.insert(s.get_rule_index());
                    }

                    return;
                }
                _ => {}
            }
        }

        for tr in s.get_transitions() {
            let target = self.atn.states[tr.get_target()].as_ref();
            match tr.get_serialization_type() {
                TransitionType::TRANSITION_RULE => {
                    let rule_tr = unsafe { cast::<RuleTransition>(tr.as_ref()) };
                    if called_rule_stack.contains(target.get_rule_index()) { continue; }

                    let new_ctx = PredictionContext::new_singleton(ctx.clone().map(Box::new), rule_tr.follow_state as isize);

                    called_rule_stack.insert(target.get_rule_index());
                    self.look_work(target, stop_state, Some(new_ctx), look, look_busy,
                                   called_rule_stack, see_thru_preds, add_eof);
                    called_rule_stack.remove(target.get_rule_index());
                }
                TransitionType::TRANSITION_PREDICATE | TransitionType::TRANSITION_PRECEDENCE => {
                    if see_thru_preds {
                        self.look_work(target, stop_state, ctx.clone(), look, look_busy,
                                       called_rule_stack, see_thru_preds, add_eof)
                    } else {
                        look.add_one(TOKEN_INVALID_TYPE)
                    }
                }
                TransitionType::TRANSITION_WILDCARD => {
                    look.add_range(TOKEN_MIN_USER_TOKEN_TYPE, self.atn.max_token_type)
                }
                _ if tr.is_epsilon() => {
                    self.look_work(target, stop_state, ctx.clone(), look, look_busy,
                                   called_rule_stack, see_thru_preds, add_eof)
                }
                _ => {
                    if let Some(mut set) = tr.get_label() {
                        if tr.get_serialization_type() == TRANSITION_NOTSET {
                            let complement = set.complement(TOKEN_MIN_USER_TOKEN_TYPE, self.atn.max_token_type);
                            *set.to_mut() = complement;
                        }
                        look.add_set(set.deref())
                    }
                }
            }
        }
    }
}
 