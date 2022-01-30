use crate::semantic_context::SemanticContext;
use crate::atn_state::{ATNState, ATNStateRef, ATNStateType};

use crate::prediction_context::PredictionContext;
use crate::atn_state::ATNBlockStart::BasicBlockStart;
use std::hash::{Hash, Hasher};
use murmur3::murmur3_32::MurmurHasher;
use crate::atn_config::ATNConfigType::LexerATNConfig;
use crate::atn_state::ATNStateType::DecisionState;
use crate::lexer_action_executor::LexerActionExecutor;
use crate::dfa::ScopeExt;

//pub trait ATNConfig: Sync + Send {
//    fn get_state(&self) -> ATNStateRef;
//    fn get_alt(&self) -> isize;
//    fn get_type(&self) -> &ATNConfigType;
//    //    fn get_semantic_context(&self) -> &SemanticContext;
//
//    fn get_context(&self) -> Option<&PredictionContext>;
//    fn take_context(&mut self) -> PredictionContext;
//    fn set_context(&mut self, v: Box<PredictionContext>);
//
//    fn get_reaches_into_outer_context(&self) -> isize;
//    fn set_reaches_into_outer_context(&mut self, v: isize);
//
//    fn get_precedence_filter_suppressed(&self) -> bool;
//    fn set_precedence_filter_suppressed(&mut self, v: bool);
//}

impl Eq for ATNConfig {}

impl PartialEq for ATNConfig {
    fn eq(&self, other: &Self) -> bool {
        self.get_state() == other.get_state() && self.get_alt() == other.get_alt()
            && self.get_context() == other.get_context()
            && self.get_type() == other.get_type()
        // && semantic context
    }
}

impl Hash for ATNConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.get_state() as i32);
        state.write_i32(self.get_alt() as i32);
        match self.get_context() {
            None => state.write_i32(0),
            Some(c) => c.hash(state),
        }
        //todo semantic context
        if let LexerATNConfig { lexer_action_executor, passed_through_non_greedy_decision } = &self.config_type {
            state.write_i32(if *passed_through_non_greedy_decision { 1 } else { 0 });
            match lexer_action_executor {
                None => state.write_i32(0),
                Some(ex) => ex.hash(state),
            }
        }
    }
}

#[derive(Clone)]
pub struct ATNConfig {
    precedence_filter_suppressed: bool,
    state: ATNStateRef,
    alt: isize,
    //todo maybe option is unnecesary and PredictionContext::EMPTY would be enough
    //another todo, maybe make separate shared PredictionContext owning structure since cloning them might be slow
    context: Option<Box<PredictionContext>>,
    pub semantic_context: Option<Box<SemanticContext>>,
    pub reaches_into_outer_context: isize,
    pub config_type: ATNConfigType,
}

#[derive(Eq, PartialEq, Clone)]
pub enum ATNConfigType {
    BaseATNConfig,
    LexerATNConfig {
        lexer_action_executor: Option<Box<LexerActionExecutor>>,
        passed_through_non_greedy_decision: bool,
    },
}

impl ATNConfig {
    pub fn get_lexer_executor(&self) -> Option<&LexerActionExecutor> {
        match &self.config_type {
            ATNConfigType::BaseATNConfig => None,
            ATNConfigType::LexerATNConfig { lexer_action_executor, .. } => lexer_action_executor.as_deref(),
        }
    }

    pub fn default_hash(&self) -> u64 {
        MurmurHasher::default().convert_with(|mut x| {
            self.hash(&mut x);
            x.finish()
        })
    }

//    fn new_base_atnconfig7(_old: &ATNConfig) -> ATNConfig {
//        unimplemented!()
//    }

    pub fn new(
        state: ATNStateRef,
        alt: isize,
        context: Option<PredictionContext>,
    ) -> ATNConfig {
        ATNConfig {
            precedence_filter_suppressed: false,
            state,
            alt,
            context: context.map(Box::new),
//            semantic_context: SemanticContext::empty(),
            semantic_context: None,
            reaches_into_outer_context: 0,
            config_type: ATNConfigType::BaseATNConfig,
        }
    }

    pub fn new_with_semantic(
        state: ATNStateRef,
        alt: isize,
        context: Option<PredictionContext>,
        semantic_context: Option<Box<SemanticContext>>,
    ) -> ATNConfig {
        let mut new = Self::new(state, alt, context);
        new.semantic_context = semantic_context;
        new
    }

    fn new_base_atnconfig4(_c: &ATNConfig, _state: &ATNState) -> ATNConfig {
        unimplemented!()
    }

    fn new_base_atnconfig3(
        _c: &ATNConfig,
        _state: &ATNState,
        _semanticContext: &SemanticContext,
    ) -> ATNConfig {
        unimplemented!()
    }

//    fn new_base_atnconfig2(_c: &ATNConfig, _semanticContext: &SemanticContext) -> ATNConfig {
//        unimplemented!()
//    }
//
//    fn new_base_atnconfig1(
//        _c: &ATNConfig,
//        _state: &ATNState,
//        _context: &PredictionContext,
//    ) -> ATNConfig {
//        unimplemented!()
//    }
//
//    fn new_base_atnconfig(
//        _c: &ATNConfig,
//        _state: &ATNState,
//        _context: &PredictionContext,
//        _semanticContext: &SemanticContext,
//    ) -> ATNConfig {
//        unimplemented!()
//    }

    pub fn new_lexer_atnconfig6(
        _state: ATNStateRef,
        _alt: isize,
        _context: PredictionContext,
    ) -> ATNConfig {
        let mut atnconfig = ATNConfig::new(_state, _alt, Some(_context));
        atnconfig.config_type = ATNConfigType::LexerATNConfig {
            lexer_action_executor: None,
            passed_through_non_greedy_decision: false,
        };
        atnconfig
    }

    //fn new_lexer_atnconfig5(state: &ATNState, alt: isize, context: PredictionContext, lexerActionExecutor: LexerActionExecutor) ->  LexerATNConfig { unimplemented!() }

    fn new_lexer_atnconfig4(_c: &ATNConfig, _state: &ATNState) -> ATNConfig {
        unimplemented!()
    }

//    pub(crate) fn new_lexer_atnconfig3(
//        c: &ATNConfig,
//        state: &ATNState, /*, lexerActionExecutor:  LexerActionExecutor*/
//    ) -> ATNConfig {
////        let prediction_context = c.get_context().map(|x| x.clone());
////        Self::new_lexer_atnconfig2(c, state, prediction_context)
//    }

    pub fn cloned_with_new_semantic(&self, target: &ATNState, ctx: Option<Box<SemanticContext>>) -> ATNConfig {
        let mut new = self.cloned(target);
        new.semantic_context = ctx;
        new
    }

    pub fn cloned(&self, target: &ATNState) -> ATNConfig {
        let mut new = self.clone();
        new.state = target.get_state_number();
        if let ATNConfigType::LexerATNConfig { passed_through_non_greedy_decision, .. } = &mut new.config_type {
            *passed_through_non_greedy_decision = check_non_greedy_decision(self, target);
        }
        new
    }

    pub fn cloned_with_new_ctx(&self, target: &ATNState, ctx: Option<PredictionContext>) -> ATNConfig {
        // todo don't full clone here since prediction context is biggest part
        let mut new = self.cloned(target);
        new.context = ctx.map(Box::new);
//        if let ATNConfigType::LexerATNConfig { passed_through_non_greedy_decision,.. } = &mut new.config_type{
//            *passed_through_non_greedy_decision = check_non_greedy_decision(self,target);
//        }

        new
    }

    pub fn cloned_with_new_exec(&self, target: &ATNState, exec: Option<LexerActionExecutor>) -> ATNConfig {
        let mut new = self.cloned(target);
        if let ATNConfigType::LexerATNConfig {
            lexer_action_executor, passed_through_non_greedy_decision
        } = &mut new.config_type {
            *lexer_action_executor = exec.map(Box::new);
//            *passed_through_non_greedy_decision = check_non_greedy_decision(self, target);
        }
        new
    }

//    pub(crate) fn new_lexer_atnconfig2(
//        c: &ATNConfig,
//        state: &ATNState,
//        _context: Option<PredictionContext>,
//    ) -> ATNConfig {
//        c.cloned_with_new_ctx(state,_context)
//    }

    fn new_lexer_atnconfig1(
        _state: &ATNState,
        _alt: isize,
        _context: &PredictionContext,
    ) -> ATNConfig {
        unimplemented!()
    }

    //fn check_non_greedy_decision(source LexerATNConfig, target: &ATNState) -> bool { unimplemented!() }
//}
//
//impl ATNConfig for BaseATNConfig {
    pub fn get_state(&self) -> ATNStateRef {
        self.state
    }

    pub fn get_alt(&self) -> isize {
        self.alt
    }

    pub fn get_type(&self) -> &ATNConfigType {
        &self.config_type
    }

    pub fn get_semantic_context(&self) -> Option<&SemanticContext> {
        self.semantic_context.as_deref()
    }

    pub fn get_context(&self) -> Option<&PredictionContext> {
        self.context.as_deref()
    }

    pub fn take_context(&mut self) -> PredictionContext {
        *self.context.take().unwrap()
    }

    pub fn set_context(&mut self, _v: Box<PredictionContext>) {
        self.context = Some(_v);
    }

    pub fn get_reaches_into_outer_context(&self) -> isize {
        self.reaches_into_outer_context
    }

    pub fn set_reaches_into_outer_context(&mut self, _v: isize) {
        self.reaches_into_outer_context = _v
    }

    pub fn get_precedence_filter_suppressed(&self) -> bool {
        self.precedence_filter_suppressed
    }

    pub fn set_precedence_filter_suppressed(&mut self, _v: bool) {
        self.precedence_filter_suppressed = _v;
    }
}

fn check_non_greedy_decision(source: &ATNConfig, target: &ATNState) -> bool {
    if let LexerATNConfig {
        passed_through_non_greedy_decision: true, ..
    } = source.get_type()
    {
        return true;
    }
    if let ATNStateType::DecisionState {
        nongreedy: true, ..
    } = target.get_state_type()
    {
        return true;
    }
    false
}

