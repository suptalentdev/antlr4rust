#![allow(non_snake_case)]

use std::any::Any;

use antlr_rust::common_token_factory::CommonTokenFactory;
// Generated from SimpleLR.g4 by ANTLR 4.8
use antlr_rust::tree::ParseTreeListener;

use super::simplelrparser::*;

pub trait SimpleLRListener<'input>: ParseTreeListener<'input, LocalTokenFactory<'input>> {
    /**
     * Enter a parse tree produced by {@link SimpleLRParser#s}.
     * @param ctx the parse tree
     */
    fn enter_s(&mut self, _ctx: &SContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link SimpleLRParser#s}.
     * @param ctx the parse tree
     */
    fn exit_s(&mut self, _ctx: &SContext<'input>) {}

    /**
     * Enter a parse tree produced by {@link SimpleLRParser#a}.
     * @param ctx the parse tree
     */
    fn enter_a(&mut self, _ctx: &AContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link SimpleLRParser#a}.
     * @param ctx the parse tree
     */
    fn exit_a(&mut self, _ctx: &AContext<'input>) {}
}
