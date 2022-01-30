#![allow(non_snake_case)]

use std::any::Any;

// Generated from Labels.g4 by ANTLR 4.8
use antlr_rust::tree::ParseTreeListener;

use super::labelsparser::*;

pub trait LabelsListener: ParseTreeListener {
    /**
     * Enter a parse tree produced by {@link LabelsParser#s}.
     * @param ctx the parse tree
     */
    fn enter_s(&mut self, _ctx: &SContext) {}
    /**
     * Exit a parse tree produced by {@link LabelsParser#s}.
     * @param ctx the parse tree
     */
    fn exit_s(&mut self, _ctx: &SContext) {}

    /**
     * Enter a parse tree produced by the {@code add}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_add(&mut self, _ctx: &AddContext) {}
    /**
     * Exit a parse tree produced by the {@code add}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_add(&mut self, _ctx: &AddContext) {}

    /**
     * Enter a parse tree produced by the {@code parens}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_parens(&mut self, _ctx: &ParensContext) {}
    /**
     * Exit a parse tree produced by the {@code parens}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_parens(&mut self, _ctx: &ParensContext) {}

    /**
     * Enter a parse tree produced by the {@code mult}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_mult(&mut self, _ctx: &MultContext) {}
    /**
     * Exit a parse tree produced by the {@code mult}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_mult(&mut self, _ctx: &MultContext) {}

    /**
     * Enter a parse tree produced by the {@code dec}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_dec(&mut self, _ctx: &DecContext) {}
    /**
     * Exit a parse tree produced by the {@code dec}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_dec(&mut self, _ctx: &DecContext) {}

    /**
     * Enter a parse tree produced by the {@code anID}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_anID(&mut self, _ctx: &AnIDContext) {}
    /**
     * Exit a parse tree produced by the {@code anID}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_anID(&mut self, _ctx: &AnIDContext) {}

    /**
     * Enter a parse tree produced by the {@code anInt}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_anInt(&mut self, _ctx: &AnIntContext) {}
    /**
     * Exit a parse tree produced by the {@code anInt}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_anInt(&mut self, _ctx: &AnIntContext) {}

    /**
     * Enter a parse tree produced by the {@code inc}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn enter_inc(&mut self, _ctx: &IncContext) {}
    /**
     * Exit a parse tree produced by the {@code inc}
     * labeled alternative in {@link LabelsParser#e}.
     * @param ctx the parse tree
     */
    fn exit_inc(&mut self, _ctx: &IncContext) {}
}
