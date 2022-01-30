use std::collections::{HashMap, LinkedList};
use std::hash::{BuildHasher, Hash, Hasher};
use murmur3::murmur3_32::MurmurHasher;
use crate::prediction_context::PredictionContext::{Singleton, Array};

pub const base_prediction_context_empty_return_state: isize = 0x7FFFFFFF;

//pub trait PredictionContext: Sync + Send {
//    fn get_parent(&self, index: isize) -> Option<&BasePredictionContext>;
//    fn get_return_state(&self, index: isize) -> isize;
//    fn length(&self) -> isize;
//    fn is_empty(&self) -> bool;
//    fn has_empty_path(&self) -> bool;
//    fn hash_code(&self)->i32;
//}


#[derive(Eq, Clone)]
pub enum PredictionContext {
    Singleton(SingletonPredictionContext),
    Array(ArrayPredictionContext),
//    Empty {
//        cached_hash: i32,
//    },
}

#[derive(Eq, PartialEq, Clone)]
pub struct ArrayPredictionContext {
    cached_hash: i32,
    parents: Vec<Option<Box<PredictionContext>>>,
    return_states: Vec<isize>,
}

#[derive(Eq, PartialEq, Clone)]
pub struct SingletonPredictionContext {
    cached_hash: i32,
    parent_ctx: Option<Box<PredictionContext>>,
    return_state: isize,
}

impl SingletonPredictionContext {
    fn is_empty(&self) -> bool {
        self.return_state == base_prediction_context_empty_return_state
    }
}

impl PartialEq for PredictionContext {
    fn eq(&self, other: &Self) -> bool {
        self.hash_code() == other.hash_code()
    }
}

impl Hash for PredictionContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code())
    }
}

lazy_static! {
    pub static ref EMPTY_PREDICTION_CONTEXT: PredictionContext =
        PredictionContext::new_empty_prediction_context();
}

impl PredictionContext {
    pub fn new(cached_hash: isize) -> PredictionContext {
        unimplemented!()
    }

    pub fn new_array_prediction_context(
        parents: Vec<Option<Box<PredictionContext>>>,
        return_states: Vec<isize>,
    ) -> PredictionContext {
        let mut ctx = PredictionContext::Array(ArrayPredictionContext {
            cached_hash: 0,
            parents,
            return_states,
        });
        ctx.calc_hash();
        ctx
    }

    pub fn new_singleton_prediction_context(
        parent_ctx: Option<Box<PredictionContext>>,
        return_state: isize,
    ) -> PredictionContext {
        let mut ctx = PredictionContext::Singleton(SingletonPredictionContext {
            cached_hash: 0,
            parent_ctx,
            return_state,
        });
        ctx.calc_hash();
        ctx
    }

    pub fn new_empty_prediction_context() -> PredictionContext {
        let mut ctx = PredictionContext::Singleton(SingletonPredictionContext {
            cached_hash: 0,
            parent_ctx: None,
            return_state: base_prediction_context_empty_return_state,
        });
        ctx.calc_hash();
        ctx
    }

    pub fn calc_hash(&mut self) {
        let mut hasher = MurmurHasher::default();
        match self {
            PredictionContext::Singleton(SingletonPredictionContext {
                                             parent_ctx,
                                             return_state,
                                             ..
                                         }) => {
                hasher.write_i32(match parent_ctx {
                    None => { 0 }
                    Some(x) => { x.hash_code() }
                });
                hasher.write_i32(*return_state as i32);
            }
            PredictionContext::Array(ArrayPredictionContext {
                                         parents,
                                         return_states,
                                         ..
                                     }) => {
                parents.iter()
                    .for_each(|x| hasher.write_i32(match x {
                        None => { 0 }
                        Some(x) => { x.hash_code() }
                    }));
                return_states.iter()
                    .for_each(|x| hasher.write_i32(*x as i32));
            }
//            PredictionContext::Empty { .. } => {}
        };

        let hash = hasher.finish() as i32;

        match self {
            PredictionContext::Singleton(SingletonPredictionContext { cached_hash, .. })
            | PredictionContext::Array(ArrayPredictionContext { cached_hash, .. })
//            | PredictionContext::Empty { cached_hash, .. }
            => *cached_hash = hash,
        };
    }
    //}
    //
    //impl PredictionContext for BasePredictionContext{
    pub fn get_parent(&self, index: usize) -> Option<&PredictionContext> {
        match self {
            PredictionContext::Singleton(singleton) => {
                assert_eq!(index, 0);
                singleton.parent_ctx.as_deref()
            }
            PredictionContext::Array(array) => {
                array.parents[index].as_deref()
            }
        }
    }

    pub fn get_return_state(&self, index: usize) -> isize {
        match self {
            PredictionContext::Singleton(SingletonPredictionContext { return_state, .. }) => {
                assert_eq!(index, 0);
                *return_state
            }
            PredictionContext::Array(ArrayPredictionContext { return_states, .. }) => return_states[index],
//            PredictionContext::Empty { .. } => {
//                assert_eq!(index, 0);
//                base_prediction_context_empty_return_state
//            }
        }
    }

    pub fn length(&self) -> usize {
        match self {
            PredictionContext::Singleton { .. } => 1,
            PredictionContext::Array(ArrayPredictionContext { return_states, .. }) => return_states.len(),
//            PredictionContext::Empty { .. } =.> 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        if let PredictionContext::Singleton(
            singleton
        ) = self {
            return singleton.is_empty();
        }
        false
    }

    pub fn has_empty_path(&self) -> bool {
        self.get_return_state(self.length() - 1) == base_prediction_context_empty_return_state
    }

    pub fn hash_code(&self) -> i32 {
        match self {
            PredictionContext::Singleton(SingletonPredictionContext { cached_hash, .. })
            | PredictionContext::Array(ArrayPredictionContext { cached_hash, .. })
//            | PredictionContext::Empty { cached_hash, .. }
            => *cached_hash,
        }
    }

    fn into_array(self) -> ArrayPredictionContext {
        match self {
            PredictionContext::Singleton(s) => {
                ArrayPredictionContext {
                    cached_hash: 0,
                    parents: vec![s.parent_ctx],
                    return_states: vec![s.return_state],
                }
            }
            PredictionContext::Array(arr) => { arr }
        }
    }

    pub fn merge(a: PredictionContext, b: PredictionContext, root_is_wildcard: bool) -> PredictionContext {
        if a == b { return a; }

        match (a, b) {
            (PredictionContext::Singleton(a), PredictionContext::Singleton(b)) => {
                Self::merge_singletons(a, b, root_is_wildcard)
            }
            (a, b) => {
                if root_is_wildcard {
                    if a.is_empty() { return Self::new_empty_prediction_context(); }
                    if b.is_empty() { return Self::new_empty_prediction_context(); }
                }

                Self::merge_arrays(a.into_array(), b.into_array(), root_is_wildcard)
            }
        }
    }

    fn merge_singletons(mut a: SingletonPredictionContext, mut b: SingletonPredictionContext, root_is_wildcard: bool/*, mergeCache: * DoubleDict*/) -> PredictionContext {
        Self::merge_root(&mut a, &mut b, root_is_wildcard).unwrap_or_else(||
            if a.return_state == b.return_state {
                let parent = Self::merge(*a.parent_ctx.clone().unwrap(), *b.parent_ctx.clone().unwrap(), root_is_wildcard);
                if Some(&parent) == a.parent_ctx.as_deref() { return Singleton(a); }
                if Some(&parent) == b.parent_ctx.as_deref() { return Singleton(b); }
                Self::new_singleton_prediction_context(Some(Box::new(parent)), a.return_state)
            } else {
                let mut result = ArrayPredictionContext {
                    cached_hash: -1,
                    parents: vec![a.parent_ctx, b.parent_ctx],
                    return_states: vec![a.return_state, b.return_state],
                };
                if !result.return_states.is_sorted() {
                    result.parents.swap(0, 1);
                    result.return_states.swap(0, 1);
                }
                Array(result)
            }
        )
    }

    fn merge_root(a: &mut SingletonPredictionContext, b: &mut SingletonPredictionContext, root_is_wildcard: bool) -> Option<PredictionContext> {
        if root_is_wildcard {
            if a.is_empty() || b.is_empty() { return Some(Self::new_empty_prediction_context()); }
        } else {
            if a.is_empty() && b.is_empty() { return Some(Self::new_empty_prediction_context()); }
            if a.is_empty() {
                return Some(Self::new_array_prediction_context(
                    vec![b.parent_ctx.take(), None],
                    vec![b.return_state, base_prediction_context_empty_return_state],
                ));
            }
            if b.is_empty() {
                return Some(Self::new_array_prediction_context(
                    vec![a.parent_ctx.take(), None],
                    vec![a.return_state, base_prediction_context_empty_return_state],
                ));
            }
        }

        None
    }

    fn merge_arrays(mut a: ArrayPredictionContext, mut b: ArrayPredictionContext, root_is_wildcard: bool/*, mergeCache: * DoubleDict*/) -> PredictionContext {
        let mut merged = ArrayPredictionContext {
            cached_hash: -1,
            parents: Vec::with_capacity(a.return_states.len() + b.return_states.len()),
            return_states: Vec::with_capacity(a.return_states.len() + b.return_states.len()),
        };
        let mut i = 0;
        let mut j = 0;

        while i < a.return_states.len() && j < b.return_states.len() {
            let a_parent = a.parents[i].take();
            let b_parent = b.parents[i].take();
            if a.return_states[i] == b.return_states[j] {
                let payload = a.return_states[i];
                let both = payload == base_prediction_context_empty_return_state
                    && a_parent.is_none() && b_parent.is_none();
                let ax_ax = a_parent.is_some() && b_parent.is_some()
                    && a_parent == b_parent;

                if both || ax_ax {
                    merged.return_states.push(payload);
                    merged.parents.push(a_parent);
                } else {
                    let merged_parent = Self::merge(*a_parent.unwrap(), *b_parent.unwrap(), root_is_wildcard);
                    merged.return_states.push(payload);
                    merged.parents.push(Some(Box::new(merged_parent)));
                }
                i += 1;
                j += 1;
            } else if a.return_states[i] < b.return_states[j] {
                merged.return_states.push(a.return_states[i]);
                merged.parents.push(a_parent);
                i += 1;
            } else {
                merged.return_states.push(b.return_states[i]);
                merged.parents.push(b_parent);
                j += 1;
            }
        }

        if i < a.return_states.len() {
            for p in i..a.return_states.len() {
                merged.parents.push(a.parents[p].take());
                merged.return_states.push(a.return_states[p]);
            }
        }
        if i < b.return_states.len() {
            for p in i..b.return_states.len() {
                merged.parents.push(b.parents[p].take());
                merged.return_states.push(b.return_states[p]);
            }
        }

        if merged.parents.len() == 1 {
            return Self::new_singleton_prediction_context(merged.parents[0].take(), merged.return_states[0]);
        }

        merged.return_states.shrink_to_fit();
        merged.parents.shrink_to_fit();

        //todo combine common parents?????

        return Array(merged);
    }

//    fn combine_common_parents(array: &mut ArrayPredictionContext) {
//
//    }
}

//    fn prediction_context_from_rule_context(a *ATN, outerContext: RuleContext) -> PredictionContext { unimplemented!() }
//

//
//
//
//    fn get_cached_base_prediction_context(context PredictionContext, contextCache: * PredictionContextCache, visited: map[PredictionContext]PredictionContext) -> PredictionContext { unimplemented!() }

pub struct PredictionContextCache {
    cache: HashMap<PredictionContext, PredictionContext, MurmurHasherBuilder>,
}

//
pub struct MurmurHasherBuilder {}

impl BuildHasher for MurmurHasherBuilder {
    type Hasher = MurmurHasher;

    fn build_hasher(&self) -> Self::Hasher {
        MurmurHasher::default()
    }
}

impl PredictionContextCache {
    pub fn new() -> PredictionContextCache {
        PredictionContextCache {
            cache: HashMap::with_hasher(MurmurHasherBuilder {}),
        }
    }

    fn add(&self, _ctx: Box<PredictionContext>) -> &PredictionContext {
        unimplemented!()
    }

    fn get(&self, _ctx: Box<PredictionContext>) -> &PredictionContext {
        unimplemented!()
    }

    fn length(&self) -> isize {
        unimplemented!()
    }
}
