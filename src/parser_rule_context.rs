use std::any::{Any, type_name, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::convert::identity;
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::errors::ANTLRError;
use crate::interval_set::Interval;
use crate::rule_context::{BaseRuleContext, CustomRuleContext, EmptyCustomRuleContext, RuleContext, Tid};
use crate::token::{OwningToken, Token};
use crate::token_factory::{CommonTokenFactory, TokenFactory};
use crate::tree::{ErrorNode, ErrorNodeCtx, ParseTree, TerminalNode, TerminalNodeCtx, Tree};

pub trait ParserRuleContext<'input>: ParseTree<'input> + RuleContext<'input> + Debug
{
    fn set_exception(&self, e: ANTLRError);

    fn set_start(&self, t: Option<<Self::TF as TokenFactory<'input>>::Tok>);

    /// Get the initial token in this context.
    /// Note that the range from start to stop is inclusive, so for rules that do not consume anything
    /// (for example, zero length or error productions) this token may exceed stop.
    ///
    fn start<'a>(&'a self) -> Ref<'a, <Self::TF as TokenFactory<'input>>::Inner> where 'input: 'a;
    fn start_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a;

    fn set_stop(&self, t: Option<<Self::TF as TokenFactory<'input>>::Tok>);
    ///
    /// Get the final token in this context.
    /// Note that the range from start to stop is inclusive, so for rules that do not consume anything
    /// (for example, zero length or error productions) this token may precede start.
    ///
    fn stop<'a>(&'a self) -> Ref<'a, <Self::TF as TokenFactory<'input>>::Inner> where 'input: 'a;
    fn stop_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a;

    fn add_token_node(&self, token: TerminalNode<'input, Self::TF>) -> ParserRuleContextType<'input, Self::TF>;
    fn add_error_node(&self, bad_token: ErrorNode<'input, Self::TF>) -> ParserRuleContextType<'input, Self::TF>;

    fn add_child(&self, child: ParserRuleContextType<'input, Self::TF>);
    fn remove_last_child(&self);

    fn enter_rule(&self, listener: &mut dyn Any);
    fn exit_rule(&self, listener: &mut dyn Any);

    fn child_of_type<T: ParserRuleContext<'input, TF=Self::TF> + 'input>(&self, pos: usize) -> Option<Rc<T>> where Self: Sized {
        let result = self.get_children().iter()
            .filter(|&it| it.self_id() == T::id())
            .nth(pos)
            .cloned();

        result.map(cast_rc)
    }

    // todo, return iterator
    fn children_of_type<T: ParserRuleContext<'input, TF=Self::TF> + 'input>(&self) -> Vec<Rc<T>> where Self: Sized {
        self.get_children()
            .iter()
            // not fully sound until `non_static_type_id` is implemented
            .filter(|&it| it.self_id() == T::id())
            .map(|it| cast_rc::<T>(it.clone()))
            .collect()
    }

    fn get_token(&self, ttype: isize, pos: usize) -> Option<Rc<TerminalNode<'input, Self::TF>>> {
        self.get_children()
            .iter()
            .filter(|&it| it.self_id() == TerminalNode::<'input, Self::TF>::id())
            .map(|it| cast_rc::<TerminalNode<'input, Self::TF>>(it.clone()))
            .filter(|it| it.symbol.borrow().get_token_type() == ttype)
            .nth(pos)
    }

    fn get_tokens(&self, ttype: isize) -> Vec<Rc<TerminalNode<'input, Self::TF>>> {
        self.get_children()
            .iter()
            .filter(|&it| it.self_id() == TerminalNode::<'input, Self::TF>::id())
            .map(|it| cast_rc::<TerminalNode<'input, Self::TF>>(it.clone()))
            .filter(|it| it.symbol.borrow().get_token_type() == ttype)
            .collect()
    }

    fn upcast(&self) -> &dyn ParserRuleContext<'input, TF=Self::TF>;
}

impl<'input, TF: TokenFactory<'input> + 'input> (dyn ParserRuleContext<'input, TF=TF> + 'input) {
    fn to_string(self: &Rc<Self>, rule_names: Option<&[&str]>, stop: Option<ParserRuleContextType<'input, TF>>) -> String {
        let mut result = String::from("[");
        let mut next: Option<Rc<Self>> = Some(self.clone());
        while let Some(ref p) = next {
            if stop.is_some() && (stop.is_none() || Rc::ptr_eq(p, stop.as_ref().unwrap())) { break }


            if let Some(rule_names) = rule_names {
                let rule_index = p.get_rule_index();
                let rule_name = rule_names.get(rule_index).map(|&it| it.to_owned())
                    .unwrap_or_else(|| rule_index.to_string());
                result.extend(rule_name.chars());
                result.push(' ');
            } else {
                if !p.is_empty() {
                    result.extend(p.get_invoking_state().to_string().chars());
                    result.push(' ');
                }
            }

            next = p.get_parent().clone();
        }
        // not optimal but we don't care here
        if result.chars().last() == Some(' ') {
            result.pop();
        }

        result.push(']');
        return result
    }
}


//requires ParserRuleContext to be Sync
//lazy_static! {
//    pub static ref EMPTY_CTX: Box<dyn ParserRuleContext> =
//        Box::new(BaseParserRuleContext::new_parser_ctx(None,-1,CustomRuleContextInternal));
//}


//todo do not calc this every time, maybe threadlocal? or it might be ok as it is because it is inlined
#[inline]
pub(crate) fn empty_ctx<'input, TF: TokenFactory<'input> + 'input>() -> Box<dyn ParserRuleContext<'input, TF=TF> + 'input> {
    Box::new(BaseParserRuleContext::new_parser_ctx(None, -1, EmptyCustomRuleContext(PhantomData)))
}

#[inline]
#[doc(hidden)]
fn cast_rc<'a, T: ParserRuleContext<'a> + 'a>(ctx: ParserRuleContextType<'a, T::TF>) -> Rc<T> {
    // not sure how safe it is
    unsafe { Rc::from_raw(Rc::into_raw(ctx) as *const T) }
}

#[inline]
#[doc(hidden)]
pub fn cast<'a, T: ParserRuleContext<'a> + 'a + ?Sized, Result: 'a>(ctx: &T) -> &Result {
    unsafe { &*(ctx as *const T as *const Result) }
}

/// should be called from generated parser only
#[inline]
#[doc(hidden)]
pub fn cast_mut<'a, T: ParserRuleContext<'a> + 'a + ?Sized, Result: 'a>(ctx: &mut Rc<T>) -> &mut Result {
//    if Rc::strong_count(ctx) != 1 { panic!("cant mutate Rc with multiple strong ref count"); }
// is it safe because parser does not save/move mutable references anywhere.
// they are only used to write data immediately in the corresponding expression
    unsafe { &mut *(Rc::get_mut_unchecked(ctx) as *mut T as *mut Result) }
}

// workaround newtype for cycle in trait definition
// i.e. you can't have `trait ParserRuleContext:BaseTrait<dyn ParserRuleContext>`
// #[derive(Clone)]
// pub struct ParseTreeNode<'input,TF:TokenFactory<'input>>(pub Rc<dyn ParserRuleContext<'input,TF=TF>>);
//
// impl<'input,TF:TokenFactory<'input>> Deref for ParseTreeNode<'input,TF>{
//     type Target = dyn ParserRuleContext<'input,TF=TF>;
//
//     fn deref(&self) -> &Self::Target {
//         self.0.deref()
//     }
// }

pub type ParserRuleContextType<'input, T> = Rc<dyn ParserRuleContext<'input, TF=T> + 'input>;
// pub type ParserRuleContextType<'input,T> = ParseTreeNode<'input,T>;

pub struct BaseParserRuleContext<'input, Ctx: CustomRuleContext<'input>> {
    base: BaseRuleContext<'input, Ctx>,

    start: RefCell<<Ctx::TF as TokenFactory<'input>>::Tok>,
    stop: RefCell<<Ctx::TF as TokenFactory<'input>>::Tok>,
    exception: Option<Box<ANTLRError>>,
    /// List of children of current node
    pub(crate) children: RefCell<Vec<ParserRuleContextType<'input, Ctx::TF>>>,
}

impl<'input, Ctx: CustomRuleContext<'input>> Debug for BaseParserRuleContext<'input, Ctx> {
    default fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(type_name::<Self>())
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> RuleContext<'input> for BaseParserRuleContext<'input, Ctx> {
    fn get_invoking_state(&self) -> isize {
        self.base.get_invoking_state()
    }

    fn set_invoking_state(&self, t: isize) {
        self.base.set_invoking_state(t)
    }

    fn get_parent_ctx(&self) -> Option<ParserRuleContextType<'input, Self::TF>> {
        self.base.get_parent_ctx()
    }

    fn set_parent(&self, parent: &Option<ParserRuleContextType<'input, Self::TF>>) {
        self.base.set_parent(parent)
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> CustomRuleContext<'input> for BaseParserRuleContext<'input, Ctx> {
    type TF = Ctx::TF;

    fn get_rule_index(&self) -> usize { self.base.ext.get_rule_index() }
}

unsafe impl<'input, Ctx: CustomRuleContext<'input>> Tid for BaseParserRuleContext<'input, Ctx> {
    fn self_id(&self) -> TypeId {
        self.base.ext.self_id()
    }

    fn id() -> TypeId where Self: Sized {
        Ctx::id()
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> Deref for BaseParserRuleContext<'input, Ctx> {
    type Target = Ctx;

    fn deref(&self) -> &Self::Target {
        &self.base.ext
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> DerefMut for BaseParserRuleContext<'input, Ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base.ext
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> Borrow<Ctx> for BaseParserRuleContext<'input, Ctx> {
    fn borrow(&self) -> &Ctx {
        &self.base.ext
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> BorrowMut<Ctx> for BaseParserRuleContext<'input, Ctx> {
    fn borrow_mut(&mut self) -> &mut Ctx {
        &mut self.base.ext
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> ParserRuleContext<'input> for BaseParserRuleContext<'input, Ctx> {
    fn set_exception(&self, _e: ANTLRError) {
        unimplemented!()
//        self.exception = Some(Box::new(e));
    }

    fn set_start(&self, t: Option<<Ctx::TF as TokenFactory<'input>>::Tok>) {
        *self.start.borrow_mut() = t.unwrap_or(Ctx::TF::create_invalid().clone());
    }

    fn start<'a>(&'a self) -> Ref<'a, <Ctx::TF as TokenFactory<'input>>::Inner> where 'input: 'a {
        Ref::map(self.start.borrow(), |t| t.borrow())
    }

    fn start_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a {
        self.start.borrow_mut()
    }

    fn set_stop(&self, t: Option<<Ctx::TF as TokenFactory<'input>>::Tok>) {
        *self.stop.borrow_mut() = t.unwrap_or(Ctx::TF::create_invalid().clone());
    }

    fn stop<'a>(&'a self) -> Ref<'a, <Ctx::TF as TokenFactory<'input>>::Inner> where 'input: 'a {
        Ref::map(self.stop.borrow(), |t| t.borrow())
    }

    fn stop_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a {
        self.stop.borrow_mut()
    }

    fn add_token_node(&self, token: TerminalNode<'input, Ctx::TF>) -> ParserRuleContextType<'input, Ctx::TF> {
        let node: ParserRuleContextType<'input, Ctx::TF> = Rc::new(token);
        self.children.borrow_mut().push(node.clone());
        node
    }

    fn add_error_node(&self, bad_token: ErrorNode<'input, Ctx::TF>) -> ParserRuleContextType<'input, Ctx::TF> {
//        bad_token.base.parent_ctx =
        let node: ParserRuleContextType<'input, Ctx::TF> = Rc::new(bad_token);
//        Backtrace::new().frames()[0].symbols()[0];

        self.children.borrow_mut().push(node.clone());
        node
    }

    fn add_child(&self, child: ParserRuleContextType<'input, Ctx::TF>) {
        self.children.borrow_mut().push(child);
    }

    fn remove_last_child(&self) {
        self.children.borrow_mut().pop();
    }

    fn enter_rule(&self, listener: &mut dyn Any) {
        Ctx::enter(self, listener)
    }

    fn exit_rule(&self, listener: &mut dyn Any) {
        Ctx::exit(self, listener)
    }

    fn upcast(&self) -> &dyn ParserRuleContext<'input, TF=Ctx::TF> {
        self
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> Tree<'input> for BaseParserRuleContext<'input, Ctx> {
    fn get_parent(&self) -> Option<ParserRuleContextType<'input, Ctx::TF>> {
        self.get_parent_ctx()
    }

    fn has_parent(&self) -> bool {
        self.base.parent_ctx.borrow().is_some()
    }

    fn get_payload(&self) -> Box<dyn Any> {
        unimplemented!()
    }

    fn get_child(&self, i: usize) -> Option<ParserRuleContextType<'input, Ctx::TF>> {
        self.children.borrow().get(i).cloned()
    }

    fn get_child_count(&self) -> usize {
        self.children.borrow().len()
    }

    fn get_children(&self) -> Ref<'_, Vec<ParserRuleContextType<'input, Ctx::TF>>> {
        self.children.borrow()
    }

    fn get_children_full(&self) -> &RefCell<Vec<ParserRuleContextType<'input, Ctx::TF>>> {
        &self.children
    }
}

impl<'input, Ctx: CustomRuleContext<'input>> ParseTree<'input> for BaseParserRuleContext<'input, Ctx> {
    fn get_source_interval(&self) -> Interval {
        Interval { a: self.start().get_token_index(), b: self.stop().get_token_index() }
    }

    default fn get_text(&self) -> String {
        let children = self.get_children();
        if children.len() == 0 {
            return String::new();
        }

        let mut result = String::new();

        for child in children.iter() {
            result += &child.get_text()
        }

        result
    }

//    fn to_string_tree(&self, r: &dyn Parser) -> String {
//
//    }
}

impl<'input, Ctx: CustomRuleContext<'input> + 'input> BaseParserRuleContext<'input, Ctx> {
    pub fn new_parser_ctx(parent_ctx: Option<ParserRuleContextType<'input, Ctx::TF>>, invoking_state: isize, ext: Ctx) -> Self {
        Self {
            base: BaseRuleContext::new_ctx(parent_ctx, invoking_state, ext),
            start: RefCell::new(Ctx::TF::create_invalid()),
            stop: RefCell::new(Ctx::TF::create_invalid()),
            exception: None,
            children: RefCell::new(vec![]),
        }
    }
    pub fn copy_from<T: ParserRuleContext<'input, TF=Ctx::TF> + ?Sized>(ctx: &T, ext: Ctx) -> Self {
        Self {
            base: BaseRuleContext::new_ctx(ctx.get_parent_ctx(), ctx.get_invoking_state(), ext),
            start: RefCell::new(ctx.start_mut().clone()),
            stop: RefCell::new(ctx.stop_mut().clone()),
            exception: None,
            children: RefCell::new(ctx.get_children().iter().cloned().collect()),
        }
    }

    pub fn to_string(self: Rc<Self>, rule_names: Option<&[&str]>, stop: Option<Rc<dyn ParserRuleContext<'input, TF=Ctx::TF>>>) -> String {
        (self as ParserRuleContextType<'input, Ctx::TF>).to_string(rule_names, stop)
    }
}


///////////////////////////////////////////////
// Needed to significantly reduce boilerplate in the generated code,
// because there is no simple way to implement trait for enum
// will not be necessary if some kind of variant types RFC will be merged
//////////////////////////////////////////////
/// workaround trait to overcome conflicting implementations error
#[doc(hidden)]
pub trait DerefSeal: Deref {}

impl<'input, T: DerefSeal<Target=I> + 'input + Debug + Tid, I: ParserRuleContext<'input> + 'input + ?Sized> ParserRuleContext<'input> for T {
    fn set_exception(&self, e: ANTLRError) { self.deref().set_exception(e) }

    fn set_start(&self, t: Option<<Self::TF as TokenFactory<'input>>::Tok>) { self.deref().set_start(t) }

    fn start<'a>(&'a self) -> Ref<'a, <Self::TF as TokenFactory<'input>>::Inner> where 'input: 'a { self.deref().start() }

    fn start_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a { self.deref().start_mut() }

    fn set_stop(&self, t: Option<<Self::TF as TokenFactory<'input>>::Tok>) { self.deref().set_stop(t) }

    fn stop<'a>(&'a self) -> Ref<'a, <Self::TF as TokenFactory<'input>>::Inner> where 'input: 'a { self.deref().stop() }

    fn stop_mut<'a>(&'a self) -> RefMut<'a, <Self::TF as TokenFactory<'input>>::Tok> where 'input: 'a { self.deref().stop_mut() }

    fn add_token_node(&self, token: TerminalNode<'input, I::TF>) -> ParserRuleContextType<'input, Self::TF> { self.deref().add_token_node(token) }

    fn add_error_node(&self, bad_token: ErrorNode<'input, I::TF>) -> ParserRuleContextType<'input, Self::TF> { self.deref().add_error_node(bad_token) }

    fn add_child(&self, child: ParserRuleContextType<'input, Self::TF>) { self.deref().add_child(child) }

    fn remove_last_child(&self) { self.deref().remove_last_child() }

    fn enter_rule(&self, listener: &mut dyn Any) { self.deref().enter_rule(listener) }

    fn exit_rule(&self, listener: &mut dyn Any) { self.deref().exit_rule(listener) }

    fn upcast(&self) -> &dyn ParserRuleContext<'input, TF=Self::TF> { self.deref().upcast() }
}

impl<'input, T: DerefSeal<Target=I> + 'input + Debug + Tid, I: ParserRuleContext<'input> + 'input + ?Sized> RuleContext<'input> for T {
    fn get_invoking_state(&self) -> isize { self.deref().get_invoking_state() }

    fn set_invoking_state(&self, t: isize) { self.deref().set_invoking_state(t) }

    fn is_empty(&self) -> bool { self.deref().is_empty() }

    fn get_parent_ctx(&self) -> Option<ParserRuleContextType<'input, Self::TF>> { self.deref().get_parent_ctx() }

    fn set_parent(&self, parent: &Option<ParserRuleContextType<'input, Self::TF>>) { self.deref().set_parent(parent) }
}

impl<'input, T: DerefSeal<Target=I> + 'input + Debug + Tid, I: ParserRuleContext<'input> + 'input + ?Sized> ParseTree<'input> for T {
    fn get_source_interval(&self) -> Interval { self.deref().get_source_interval() }

    fn get_text(&self) -> String { self.deref().get_text() }
}

impl<'input, T: DerefSeal<Target=I> + 'input + Debug + Tid, I: ParserRuleContext<'input> + 'input + ?Sized +> Tree<'input> for T {
    fn get_parent(&self) -> Option<ParserRuleContextType<'input, Self::TF>> { self.deref().get_parent() }

    fn has_parent(&self) -> bool { self.deref().has_parent() }

    fn get_payload(&self) -> Box<dyn Any> { self.deref().get_payload() }

    fn get_child(&self, i: usize) -> Option<ParserRuleContextType<'input, Self::TF>> { self.deref().get_child(i) }

    fn get_child_count(&self) -> usize { self.deref().get_child_count() }

    fn get_children(&self) -> Ref<'_, Vec<ParserRuleContextType<'input, Self::TF>>> { self.deref().get_children() }

    fn get_children_full(&self) -> &RefCell<Vec<ParserRuleContextType<'input, Self::TF>>> { self.deref().get_children_full() }
}

impl<'input, T: DerefSeal<Target=I> + 'input + Debug + Tid, I: ParserRuleContext<'input> + 'input + ?Sized> CustomRuleContext<'input> for T {
    type TF = I::TF;

    fn get_rule_index(&self) -> usize { self.deref().get_rule_index() }

    // fn type_rule_index() -> usize where Self: Sized { unimplemented!() }

    fn get_alt_number(&self) -> isize { self.deref().get_alt_number() }

    fn set_alt_number(&self, _alt_number: isize) { self.deref().set_alt_number(_alt_number) }
}

//
//    fn get_text(&self) -> String { unimplemented!() }
//
//    fn add_terminal_node_child(&self, child: TerminalNode) -> TerminalNode { unimplemented!() }
//
//    fn get_child_of_type(&self, i: isize, childType: reflect.Type) -> RuleContext { unimplemented!() }
//
//    fn to_string_tree(&self, ruleNames Vec<String>, recog: Recognizer) -> String { unimplemented!() }
//
//    fn get_rule_context(&self) -> RuleContext { unimplemented!() }
//
//    fn accept(&self, visitor: ParseTreeVisitor) -> interface { unimplemented!() } {
//    return visitor.VisitChildren(prc)
//    }
//
//    fn get_token(&self, ttype: isize, i: isize) -> TerminalNode { unimplemented!() }
//
//    fn get_tokens(&self, ttype: isize) -> Vec<TerminalNode> { unimplemented!() }
//
//    fn get_payload(&self) -> interface { unimplemented!() } {
//    return: prc,
//    }
//
//    fn get_child(&self, ctxType: reflect.Type, i: isize) -> RuleContext { unimplemented!() }
//
//
//    fn get_typed_rule_context(&self, ctxType: reflect.Type, i: isize) -> RuleContext { unimplemented!() }
//
//    fn get_typed_rule_contexts(&self, ctxType: reflect.Type) -> Vec<RuleContext> { unimplemented!() }
//
//    fn get_child_count(&self) -> int { unimplemented!() }
//
//    fn get_source_interval(&self) -> * Interval { unimplemented!() }
//
//
//    fn String(&self, ruleNames Vec<String>, stop: RuleContext) -> String { unimplemented!() }
//
//    var RuleContextEmpty = NewBaseParserRuleContext(nil, - 1)
//
//    pub trait InterpreterRuleContext {
//    parser_rule_context
//    }
//
//    pub struct BaseInterpreterRuleContext {
//    base: BaseParserRuleContext,
//    }
//
//    fn new_base_interpreter_rule_context(parent BaseInterpreterRuleContext, invokingStateNumber: isize, ruleIndex: isize) -> * BaseInterpreterRuleContext { unimplemented!() }
