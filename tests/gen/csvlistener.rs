#![allow(non_snake_case)]

use std::any::Any;

use antlr_rust::token_factory::CommonTokenFactory;
// Generated from CSV.g4 by ANTLR 4.8
use antlr_rust::tree::ParseTreeListener;

use super::csvparser::*;

pub trait CSVListener<'input>: ParseTreeListener<'input, LocalTokenFactory<'input>> {
    /**
     * Enter a parse tree produced by {@link CSVParser#csvFile}.
     * @param ctx the parse tree
     */
    fn enter_csvFile(&mut self, _ctx: &CsvFileContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CSVParser#csvFile}.
     * @param ctx the parse tree
     */
    fn exit_csvFile(&mut self, _ctx: &CsvFileContext<'input>) {}

    /**
     * Enter a parse tree produced by {@link CSVParser#hdr}.
     * @param ctx the parse tree
     */
    fn enter_hdr(&mut self, _ctx: &HdrContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CSVParser#hdr}.
     * @param ctx the parse tree
     */
    fn exit_hdr(&mut self, _ctx: &HdrContext<'input>) {}

    /**
     * Enter a parse tree produced by {@link CSVParser#row}.
     * @param ctx the parse tree
     */
    fn enter_row(&mut self, _ctx: &RowContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CSVParser#row}.
     * @param ctx the parse tree
     */
    fn exit_row(&mut self, _ctx: &RowContext<'input>) {}

    /**
     * Enter a parse tree produced by {@link CSVParser#field}.
     * @param ctx the parse tree
     */
    fn enter_field(&mut self, _ctx: &FieldContext<'input>) {}
    /**
     * Exit a parse tree produced by {@link CSVParser#field}.
     * @param ctx the parse tree
     */
    fn exit_field(&mut self, _ctx: &FieldContext<'input>) {}
}
