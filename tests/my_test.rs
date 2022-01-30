#![feature(try_blocks)]
#![feature(inner_deref)]
//! Integration tests

#[macro_use]
extern crate lazy_static;

mod gen {
    use std::fmt::Write;
    use std::io::Read;
    use std::iter::FromIterator;

    use antlr_rust::common_token_factory::{ArenaCowFactory, CommonTokenFactory};
    use antlr_rust::common_token_stream::CommonTokenStream;
    use antlr_rust::input_stream::InputStream;
    use antlr_rust::int_stream::IntStream;
    use antlr_rust::lexer::Lexer;
    use antlr_rust::parser_rule_context::{BaseParserRuleContext, ParserRuleContext};
    use antlr_rust::token::{Token, TOKEN_EOF};
    use antlr_rust::token_stream::{TokenStream, UnbufferedTokenStream};
    use antlr_rust::tree::{ParseTree, ParseTreeListener, TerminalNode, TerminalNodeCtx};
    use csvlexer::*;
    use csvlistener::*;
    use csvparser::CSVParser;
    use referencetoatnlexer::ReferenceToATNLexer;
    use referencetoatnlistener::ReferenceToATNListener;
    use referencetoatnparser::ReferenceToATNParser;
    use xmllexer::XMLLexer;

    use crate::gen::labelslexer::LabelsLexer;
    use crate::gen::labelsparser::{AddContext, EContextAll, LabelsParser};
    use crate::gen::simplelrlexer::SimpleLRLexer;
    use crate::gen::simplelrlistener::SimpleLRListener;
    use crate::gen::simplelrparser::SimpleLRParser;

    mod csvlexer;
    mod csvparser;
    mod csvlistener;
    mod xmllexer;
    mod referencetoatnparser;
    mod referencetoatnlexer;
    mod referencetoatnlistener;
    mod simplelrparser;
    mod simplelrlexer;
    mod simplelrlistener;

    #[test]
    fn lexer_test_xml() -> std::io::Result<()> {
        let data =
            r#"<?xml version="1.0"?>
<!--comment-->>
<?xml-stylesheet type="text/css" href="nutrition.css"?>
<script>
<![CDATA[
function f(x) {
if (x < x && a > 0) then duh
}
]]>
</script>"#.to_owned();
        let mut _lexer = XMLLexer::new(Box::new(InputStream::new(&data)));
        //        _lexer.base.add_error_listener();
        let _a = "a".to_owned() + "";
        let mut string = String::new();
        {
            let mut token_source = UnbufferedTokenStream::new_unbuffered(&mut _lexer);
            while token_source.la(1) != TOKEN_EOF {
                {
                    let token = token_source.lt(1).unwrap();

                    let len = token.get_stop() as usize + 1 - token.get_start() as usize;
                    string.extend(
                        format!("{},len {}:\n{}\n",
                                xmllexer::_SYMBOLIC_NAMES[token.get_token_type() as usize].unwrap_or(&format!("{}", token.get_token_type())),
                                len,
                                String::from_iter(data.chars().skip(token.get_start() as usize).take(len))
                        ).chars());
                }
                token_source.consume();
            }
        }
        println!("{}", string);
        println!("{}", _lexer.get_interpreter().unwrap().get_dfa().to_lexer_string());
        Ok(())
    }

    #[test]
    fn lexer_test_csv() {
        println!("test started lexer_test_csv");
        let tf = ArenaCowFactory::default();
        let mut _lexer = CSVLexer::new_with_token_factory(
            Box::new(InputStream::new("V123,V2\nd1,d2".into())),
            &tf,
        );
        let mut token_source = CommonTokenStream::new(_lexer);
        let mut token_source_iter = token_source.iter();
        assert_eq!(token_source_iter.next().unwrap(), 5);
        assert_eq!(token_source_iter.next().unwrap(), 1);
        assert_eq!(token_source_iter.next().unwrap(), 5);
        assert_eq!(token_source_iter.next().unwrap(), 3);
        assert_eq!(token_source_iter.next().unwrap(), 5);
        assert_eq!(token_source_iter.next().unwrap(), 1);
        assert_eq!(token_source_iter.next().unwrap(), 5);
        assert_eq!(token_source_iter.next(), None);
    }

    struct Listener {}

    impl<'input> ParseTreeListener<'input, ArenaCowFactory<'input>> for Listener {
        fn enter_every_rule(&mut self, ctx: &dyn ParserRuleContext<'input, TF=ArenaCowFactory<'input>>) {
            println!("rule entered {}", csvparser::ruleNames.get(ctx.get_rule_index()).unwrap_or(&"error"))
        }
    }

    impl<'input> CSVListener<'input> for Listener {}

    #[test]
    fn parser_test_csv() {
        println!("test started");
        let tf = ArenaCowFactory::default();
        let mut _lexer = CSVLexer::new_with_token_factory(
            Box::new(InputStream::new("V123,V2\nd1,d2".into())),
            &tf,
        );
        let token_source = CommonTokenStream::new(_lexer);
        let mut parser = CSVParser::new(Box::new(token_source));
        parser.add_parse_listener(Box::new(Listener {}));
        println!("\nstart parsing parser_test_csv");
        let result = parser.csvFile();
        assert!(result.is_ok())
    }

    struct Listener2 {}

    impl<'input> ParseTreeListener<'input, CommonTokenFactory> for Listener2 {
        fn enter_every_rule(&mut self, ctx: &dyn ParserRuleContext<'input, TF=CommonTokenFactory>) {
            println!("rule entered {}", referencetoatnparser::ruleNames.get(ctx.get_rule_index()).unwrap_or(&"error"))
        }
    }

    impl<'input> ReferenceToATNListener<'input> for Listener2 {}

    #[test]
    fn adaptive_predict_test() {
        let mut _lexer = ReferenceToATNLexer::new(Box::new(InputStream::new("a 34 b".into())));
        let token_source = CommonTokenStream::new(_lexer);
        let mut parser = ReferenceToATNParser::new(Box::new(token_source));
        parser.add_parse_listener(Box::new(Listener2 {}));
        println!("\nstart parsing adaptive_predict_test");
        let result = parser.a();
        assert!(result.is_ok())
    }

    struct Listener3;

    impl<'input> ParseTreeListener<'input, CommonTokenFactory> for Listener3 {
        fn visit_terminal(&mut self, node: &TerminalNode<'input, CommonTokenFactory>) {
            println!("terminal node {}", node.symbol.get_text());
        }

        fn enter_every_rule(&mut self, ctx: &dyn ParserRuleContext<'input, TF=CommonTokenFactory>) {
            println!("rule entered {}", simplelrparser::ruleNames.get(ctx.get_rule_index()).unwrap_or(&"error"))
        }

        fn exit_every_rule(&mut self, ctx: &dyn ParserRuleContext<'input, TF=CommonTokenFactory>) {
            println!("rule exited {}", simplelrparser::ruleNames.get(ctx.get_rule_index()).unwrap_or(&"error"))
        }
    }

    impl<'input> SimpleLRListener<'input> for Listener3 {}

    #[test]
    fn lr_test() {
        // let mut
        let mut _lexer = SimpleLRLexer::new(Box::new(InputStream::new("x y z".into())));
        let token_source = CommonTokenStream::new(_lexer);
        let mut parser = SimpleLRParser::new(Box::new(token_source));
        parser.add_parse_listener(Box::new(Listener3));
        println!("\nstart parsing lr_test");
        let result = parser.s().expect("expected to parse successfully");
        assert_eq!(result.to_string_tree(&*parser), "(s (a (a (a x) y) z))");
    }

    struct Listener4 { data: String }

    impl<'input> ParseTreeListener<'input, CommonTokenFactory> for Listener4 {
        fn visit_terminal(&mut self, node: &TerminalNode<'input, CommonTokenFactory>) {
            writeln!(&mut self.data, "terminal node {}", node.symbol.get_text());
        }
    }

    impl<'input> SimpleLRListener<'input> for Listener4 {}

    #[test]
    fn test_remove_listener() {
        let mut _lexer = SimpleLRLexer::new(Box::new(InputStream::new("x y z".into())));
        let token_source = CommonTokenStream::new(_lexer);
        let mut parser = SimpleLRParser::new(Box::new(token_source));
        parser.add_parse_listener(Box::new(Listener3));
        let id = parser.add_parse_listener(Box::new(Listener4 { data: String::new() }));
        let _result = parser.s().expect("expected to parse successfully");

        let listener = parser.remove_parse_listener(id);
        assert_eq!(&listener.data, "terminal node x\nterminal node y\nterminal node z\n");
    }


    mod labelslexer;
    mod labelsparser;
    mod labelslistener;

    #[test]
    fn test_complex_convert() {
        let mut lexer = LabelsLexer::new(Box::new(InputStream::new("(a+4)*2".into())));
        let token_source = CommonTokenStream::new(lexer);
        let mut parser = LabelsParser::new(Box::new(token_source));
        let result = parser.s().expect("parser error");
        let string = result.q.as_ref().unwrap().get_v();
        assert_eq!("* + a 4 2", string);
        let x = result.q.as_deref().unwrap();
        match x {
            EContextAll::MultContext(x) => { assert_eq!("(a+4)", x.a.as_ref().unwrap().get_text()) },
            _ => panic!("oops")
        }
    }
}