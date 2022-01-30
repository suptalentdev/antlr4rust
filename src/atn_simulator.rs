use std::sync::Arc;

use crate::atn::ATN;
use crate::dfa::DFA;
use crate::prediction_context::PredictionContext;
use crate::prediction_context::PredictionContextCache;

pub trait IATNSimulator {
    fn shared_context_cache(&self) -> Arc<PredictionContextCache>;
    fn atn(&self) -> &ATN;
    fn decision_to_dfa(&self) -> &Vec<DFA>;
}

pub struct BaseATNSimulator {
    pub atn: Arc<ATN>,
    pub shared_context_cache: Arc<PredictionContextCache>,
    pub decision_to_dfa: Arc<Vec<DFA>>,
}

impl BaseATNSimulator {
    pub fn new_base_atnsimulator(
        atn: Arc<ATN>,
        decision_to_dfa: Arc<Vec<DFA>>,
        shared_context_cache: Arc<PredictionContextCache>,
    ) -> BaseATNSimulator {
        BaseATNSimulator {
            atn,
            shared_context_cache,
            decision_to_dfa,
        }
    }

    fn get_cached_context(&self, _context: Box<PredictionContext>) -> &PredictionContext {
        unimplemented!()
    }
}

impl IATNSimulator for BaseATNSimulator {
    fn shared_context_cache(&self) -> Arc<PredictionContextCache> {
        self.shared_context_cache.clone()
    }

    fn atn(&self) -> &ATN {
        self.atn.as_ref()
    }

    fn decision_to_dfa(&self) -> &Vec<DFA> {
        self.decision_to_dfa.as_ref()
    }
}
