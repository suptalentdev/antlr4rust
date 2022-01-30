use crate::semantic_context::SemanticContext;
use crate::atn_config::ATNConfig;
use std::collections::{HashMap, HashSet};
use crate::prediction_context::{MurmurHasherBuilder, PredictionContext};
use std::hash::{Hash, Hasher};
use std::cmp::max;

//pub trait ATNConfigSet:Sync+Send{
//    fn hash(&self) ->isize;
//    fn add_cached(&mut self, config: Box<dyn ATNConfig>, merge_cache: Option<&HashMap<(PredictionContext,PredictionContext),PredictionContext>>) -> bool;
//    fn add(&mut self, config: Box<dyn ATNConfig>) -> &dyn ATNConfig;
//    fn add_all(&mut self, coll: Vec<&dyn ATNConfig>) -> bool;
//
////    fn get_states(&self) -> * Set;
//    fn get_predicates(&self) -> Vec<&dyn SemanticContext>;
//    fn get_items<T:Iterator<Item=&dyn ATNConfig>>(&self) -> T;
//
////    fn optimize_configs(&self, interpreter: &BaseATNSimulator);
//
//    fn equals(&self, other: &dyn ATNConfigSet) ->bool;
//
//    fn length(&self) -> isize;
//    fn is_empty(&self) -> bool;
//    fn contains(&self, item: &dyn ATNConfig) -> bool;
//    fn contains_fast(&self, item: &dyn ATNConfig) -> bool;
//    fn clear(&self);
//    fn String(&self) -> String;
//
//    fn has_semantic_context(&self) -> bool;
//    fn set_has_semantic_context(&mut self, v: bool);
//
//    fn read_only(&self) -> bool;
//    fn set_read_only(&self, readOnly: bool);
//
////    fn get_conflicting_alts(&self) -> * BitSet;
////    fn set_conflicting_alts(&self, v: * BitSet);
//
//    fn full_context(&self) -> bool;
//
//    fn get_unique_alt(&self) -> isize;
//    fn set_unique_alt(&self, v: isize);
//
//    fn get_dips_into_outer_context(&self) -> bool;
//    fn set_dips_into_outer_context(&self, v: bool);
//}

#[derive(Eq, PartialEq)]
pub struct ATNConfigSet {
    cached_hash: isize,

    config_lookup: HashMap<u64, usize>,

    configs: Vec<Box<dyn ATNConfig>>,

    //    conflicting_alts: * BitSet,
    dips_into_outer_context: bool,

    full_ctx: bool,

    has_semantic_context: bool,

    read_only: bool,

    unique_alt: isize,
}

impl Hash for ATNConfigSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.configs.hash(state);
    }
}

impl ATNConfigSet {
    pub fn new_base_atnconfig_set(full_ctx: bool) -> ATNConfigSet {
        ATNConfigSet {
            cached_hash: -1,
            config_lookup: HashMap::new(),
            configs: vec![],
            dips_into_outer_context: false,
            full_ctx,
            has_semantic_context: false,
            read_only: false,
            unique_alt: 0,
        }
    }

    fn hash_code_configs(&self) -> isize {
        unimplemented!()
    }

    fn new_ordered_atnconfig_set() -> ATNConfigSet {
        let a = ATNConfigSet::new_base_atnconfig_set(true);
        //        a.config_lookup =
        unimplemented!();
        a
    }

    fn equal_atnconfigs() -> bool {
        unimplemented!()
    }
    //}
    //
    //impl ATNConfigSet for BaseATNConfigSet {

    //    fn add(&self, config: ATNConfig, mergeCache: * DoubleDict) -> bool { unimplemented!() }
    fn atn_config_local_hash(config: &dyn ATNConfig) -> u64 {
        let mut hashcode = 7;
        hashcode = 31 * hashcode + config.get_state();
        hashcode = 31 * hashcode + config.get_alt() as usize;
        //todo semantic context
//        hashcode = 31* hashcode + config

        hashcode as u64
    }

    pub fn add_cached(
        &mut self,
        mut config: Box<ATNConfig>,
        merge_cache: Option<&HashMap<(PredictionContext, PredictionContext), PredictionContext>>,
    ) -> bool {
        assert!(!self.read_only);
        //todo semantic context

        if config.get_reaches_into_outer_context() > 0 {
            self.dips_into_outer_context = true
        }
        let hash = Self::atn_config_local_hash(config.as_ref());

        if let Some(existing) = self.config_lookup.get(&hash) {
            let existing = self.configs.get_mut(*existing).unwrap().as_mut();
            let root_is_wildcard = !self.full_ctx;
            let mut merged = PredictionContext::merge(
                existing.take_context(),
                config.take_context(),
                root_is_wildcard,
            );
            merged.calc_hash();

            existing.set_reaches_into_outer_context(
                max(existing.get_reaches_into_outer_context(), config.get_reaches_into_outer_context())
            );

            if config.get_precedence_filter_suppressed() {
                existing.set_precedence_filter_suppressed(true)
            }

            existing.set_context(Box::new(merged));
        } else {
            self.config_lookup.insert(hash, self.configs.len());
            self.cached_hash = -1;
            self.configs.push(config);
        }
        true
    }

    //    pub fn get_states(&self) -> * Set { unimplemented!() }

    pub fn add(&mut self, config: Box<ATNConfig>) -> bool {
        self.add_cached(config, None)
    }

    pub fn add_all(&mut self, _coll: Vec<&ATNConfig>) -> bool {
        unimplemented!()
    }

    pub fn get_predicates(&self) -> Vec<&SemanticContext> {
        unimplemented!()
    }

    pub fn get_items(&self) -> impl Iterator<Item=&ATNConfig> {
        self.configs.iter().map(|c| c.as_ref())
    }

    //    pub fn optimize_configs(&self, interpreter: &BaseATNSimulator) { unimplemented!() }

    pub fn equals(&self, _other: &ATNConfigSet) -> bool {
        unimplemented!()
    }

    pub fn length(&self) -> usize {
        self.configs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }

    pub fn contains(&self, _item: &ATNConfig) -> bool {
        unimplemented!()
    }

    pub fn contains_fast(&self, _item: &ATNConfig) -> bool {
        unimplemented!()
    }

    pub fn clear(&self) {
        unimplemented!()
    }

    pub fn String(&self) -> String {
        unimplemented!()
    }

    pub fn has_semantic_context(&self) -> bool {
        self.has_semantic_context
    }

    pub fn set_has_semantic_context(&mut self, _v: bool) {
        self.has_semantic_context = _v;
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, _read_only: bool) {
        self.read_only = _read_only;
    }

    pub fn full_context(&self) -> bool {
        unimplemented!()
    }

    //    pub fn get_conflicting_alts(&self) -> * BitSet { unimplemented!() }

    //    pub fn set_conflicting_alts(&self, v: * BitSet) { unimplemented!() }

    pub fn get_unique_alt(&self) -> isize {
        unimplemented!()
    }

    pub fn set_unique_alt(&self, _v: isize) {
        unimplemented!()
    }

    pub fn get_dips_into_outer_context(&self) -> bool {
        unimplemented!()
    }

    pub fn set_dips_into_outer_context(&self, _v: bool) {
        unimplemented!()
    }
}
