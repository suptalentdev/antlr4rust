use std::borrow::Cow;
use std::borrow::Cow::Owned;
use std::cell::{Cell, RefCell};
use std::ops::DerefMut;
use std::rc::Rc;

use crate::char_stream::CharStream;
use crate::common_token_factory::{CommonTokenFactory, TokenFactory};
use crate::error_listener::{ConsoleErrorListener, ErrorListener};
use crate::errors::ANTLRError;
use crate::lexer_atn_simulator::{ILexerATNSimulator, LexerATNSimulator};
use crate::parser_rule_context::ParserRuleContext;
use crate::recognizer::{Actions, Recognizer};
use crate::token::{Token, TOKEN_INVALID_TYPE};
use crate::token_source::TokenSource;

pub trait Lexer<'input>: TokenSource<'input> + Recognizer {
    /// Sets channel where current token will be pushed
    ///
    /// By default two channels are available:
    ///  - `LEXER_DEFAULT_TOKEN_CHANNEL`
    ///  - `LEXER_HIDDEN`
    fn set_channel(&mut self, v: isize);

    /// Pushes current mode to internal mode stack and sets `m` as current lexer mode
    /// `pop_mode should be used to recover previous mode
    fn push_mode(&mut self, m: usize);

    /// Pops mode from internal mode stack
    fn pop_mode(&mut self) -> Option<usize>;

    /// Sets type of the current token
    /// Called from action to override token that will be emitted by lexer
    fn set_type(&mut self, t: isize);

    /// Sets lexer mode discarding current one
    fn set_mode(&mut self, m: usize);

    /// Used to informs lexer that it should consider next token as a continution of the current one
    fn more(&mut self);

    /// Tells lexer to completely ignore and not emit current token.
    fn skip(&mut self);

    fn reset(&mut self);

    fn get_interpreter(&self) -> Option<&LexerATNSimulator>;
}

/// **! Usually generated by ANTLR !**
///
/// This trait combines everything that can be used to extend Lexer behavior
pub trait LexerRecog<T>: Recognizer + Actions<T> + Sized + 'static {
    /// Callback to extend emit behavior
    fn before_emit(_lexer: &mut T) {}
}

pub struct BaseLexer<'input,
    T: LexerRecog<Self> + 'static,
    TF: TokenFactory<'input> = CommonTokenFactory
> {
    pub interpreter: Option<LexerATNSimulator>,
    pub input: Option<Box<dyn CharStream<'input>>>,
    recog: T,

    factory: &'input TF,

    error_listeners: RefCell<Vec<Box<dyn ErrorListener>>>,

    pub token_start_char_index: isize,
    pub token_start_line: isize,
    pub token_start_column: isize,
    current_pos: Rc<LexerPosition>,
    pub token_type: isize,
    pub token: Option<TF::Tok>,
    hit_eof: bool,
    pub channel: isize,
    mode_stack: Vec<usize>,
    pub mode: usize,
    pub text: Option<String>,
}

pub(crate) struct LexerPosition {
    pub(crate) line: Cell<isize>,
    pub(crate) char_position_in_line: Cell<isize>,
}

impl<'input, T, TF> Recognizer for BaseLexer<'input, T, TF>
    where T: LexerRecog<Self> + 'static,
          TF: TokenFactory<'input>
{
    fn sempred(&mut self, _localctx: &dyn ParserRuleContext, rule_index: isize, action_index: isize) -> bool {
        <T as Actions<Self>>::sempred(_localctx, rule_index, action_index, self)
    }

    fn action(&mut self, _localctx: &dyn ParserRuleContext, rule_index: isize, action_index: isize) {
        <T as Actions<Self>>::action(_localctx, rule_index, action_index, self)
    }
}

pub const LEXER_DEFAULT_MODE: usize = 0;
pub const LEXER_MORE: isize = -2;
pub const LEXER_SKIP: isize = -3;

pub const LEXER_DEFAULT_TOKEN_CHANNEL: isize = super::token::TOKEN_DEFAULT_CHANNEL;
pub const LEXER_HIDDEN: isize = super::token::TOKEN_HIDDEN_CHANNEL;
pub const LEXER_MIN_CHAR_VALUE: isize = 0x0000;
pub const LEXER_MAX_CHAR_VALUE: isize = 0x10FFFF;

impl<'input, 'tokens, T, TF> BaseLexer<'input, T, TF>
    where T: LexerRecog<Self> + 'static,
          TF: TokenFactory<'input>
{
    fn emit_token(&mut self, token: TF::Tok) {
        self.token = Some(token);
    }

    fn emit(&mut self) {
        <T as LexerRecog<Self>>::before_emit(self);
        let stop = self.get_char_index() - 1;
        let token = self.factory.create(
            Some(self.input.as_mut().unwrap().as_mut()),
            self.token_type,
            self.text.take(),
            self.channel,
            self.token_start_char_index,
            stop,
            self.token_start_line,
            self.token_start_column,
        );
        self.emit_token(token);
    }

    fn emit_eof(&mut self) {
        let token = self.factory.create(
            None,
            super::int_stream::EOF,
            None,
            LEXER_DEFAULT_TOKEN_CHANNEL,
            self.get_char_index(),
            self.get_char_index() - 1,
            self.get_line(),
            self.get_char_position_in_line(),
        );
        self.emit_token(token)
    }

    pub fn get_type(&self) -> isize {
        self.token_type
    }

    pub fn get_char_index(&self) -> isize {
        self.input.as_ref().unwrap().index()
    }

    pub fn get_text(&self) -> &'input str {
        self.input.as_ref().unwrap().get_text(self.token_start_char_index, self.get_char_index() - 1)
    }

    /// Used from lexer actions to override token text
    pub fn set_text(&mut self, _text: String) {
        self.text = Some(_text);
    }

    fn get_all_tokens(&self) -> Vec<Box<dyn Token>> {
        unimplemented!()
    }

    fn get_error_display_for_char(&self, _c: char) -> String {
        unimplemented!()
    }

    fn get_char_error_display(&self, _c: char) -> String {
        unimplemented!()
    }

    /// Add error listener
    pub fn add_error_listener(&mut self, listener: Box<dyn ErrorListener>) {
        self.error_listeners.borrow_mut().push(listener);
    }

    pub fn remove_error_listeners(&mut self) {
        self.error_listeners.borrow_mut().clear();
    }

    pub fn new_base_lexer(
        input: Box<dyn CharStream<'input>>,
        interpreter: LexerATNSimulator,
        recog: T,
        factory: &'input TF,
    ) -> Self {
        let mut lexer = Self {
            interpreter: Some(interpreter),
            input: Some(input),
            recog,
            factory,
            error_listeners: RefCell::new(vec![Box::new(ConsoleErrorListener {})]),
            token_start_char_index: 0,
            token_start_line: 0,
            token_start_column: 0,
            current_pos: Rc::new(LexerPosition { line: Cell::new(1), char_position_in_line: Cell::new(0) }),
            token_type: super::token::TOKEN_INVALID_TYPE,
            text: None,
            token: None,
            hit_eof: false,
            channel: super::token::TOKEN_DEFAULT_CHANNEL,
            //            token_factory_source_pair: None,
            mode_stack: Vec::new(),
            mode: self::LEXER_DEFAULT_MODE,
        };
        let pos = lexer.current_pos.clone();
        lexer.interpreter.as_mut().unwrap().current_pos = pos;
        lexer
    }
}

impl<'input, T, TF> TokenSource<'input> for BaseLexer<'input, T, TF>
    where T: LexerRecog<Self> + 'static,
          TF: TokenFactory<'input>
{
    type TF = TF;

    #[allow(unused_labels)]
    fn next_token(&mut self) -> <Self::TF as TokenFactory<'input>>::Tok {
        assert!(self.input.is_some());

        let _marker = self.input.as_mut().unwrap().mark();
        'outer: loop {
            if self.hit_eof {
                self.emit_eof();
                break;
            }
            self.token = None;
            self.channel = LEXER_DEFAULT_TOKEN_CHANNEL;
            self.token_start_column = self.interpreter.as_ref().unwrap().get_char_position_in_line();
            self.token_start_line = self.interpreter.as_ref().unwrap().get_line();
            self.text = None;
            let index = self.get_input_stream().unwrap().index();
            self.token_start_char_index = index;

            'inner: loop {
                let ttype;
                self.token_type = TOKEN_INVALID_TYPE;
                {
                    // detach from self, to allow self to be passed deeper
                    let mut interpreter = self.interpreter.take().unwrap();
//                    let mut input = self.input.take().unwrap();
                    let result = interpreter
                        .match_token(self.mode, self);
                    self.interpreter = Some(interpreter);

                    ttype = match result {
                        Ok(ttype) => {
//                            println!("new mode {}",self.mode);
                            ttype
                        },
                        Err(err) => {
//                            println!("error, recovering");
                            notify_listeners(&mut self.error_listeners.borrow_mut(), &err, self);
                            self.interpreter.as_mut().unwrap().recover(err, self.input.as_mut().unwrap().deref_mut());
                            LEXER_SKIP
                        }
                    };
//                    self.input = Some(input)
                }
                if self.get_input_stream().unwrap().la(1) == super::int_stream::EOF {
                    self.hit_eof = true;
                }

                if self.token_type == TOKEN_INVALID_TYPE {
                    self.token_type = ttype;
                }

                if self.token_type == LEXER_SKIP {
                    continue 'outer;
                }

                if self.token_type != LEXER_MORE {
                    break;
                }
            }

            if self.token.is_none() {
                self.emit();
                break;
            }
        }
        self.input.as_mut().unwrap().release(_marker);
        self.token.take().unwrap()
    }

    fn get_line(&self) -> isize {
        self.current_pos.line.get()
    }

    fn get_char_position_in_line(&self) -> isize {
        self.current_pos.char_position_in_line.get()
    }

    fn get_input_stream(&mut self) -> Option<&mut dyn CharStream<'input>> {
        match self.input {
            None => None,
            Some(ref mut x) => { Some(x.deref_mut()) },
        }
    }

    fn get_source_name(&self) -> String {
        self.input.as_deref().map(|it| it.get_source_name()).unwrap_or("<none>".to_string())
    }

//    fn set_token_factory<'c: 'b>(&mut self, f: &'c TokenFactory) {
//        self.factory = f;
//    }

    fn get_token_factory(&self) -> &'input TF {
        self.factory
    }
}

fn notify_listeners<'input, T, TF>(_liseners: &mut Vec<Box<dyn ErrorListener>>, e: &ANTLRError, lexer: &BaseLexer<'input, T, TF>)
    where T: LexerRecog<BaseLexer<'input, T, TF>> + 'static,
          TF: TokenFactory<'input>
{
    let text = format!("token recognition error at: '{}'", lexer.input.as_ref().unwrap().get_text(lexer.token_start_char_index, lexer.get_char_index()));
    for listener in _liseners.iter_mut() {
        listener.syntax_error(None, lexer.token_start_line, lexer.token_start_column, &text, Some(e))
    }
}


impl<'input, T, TF> Lexer<'input> for BaseLexer<'input, T, TF>
    where T: LexerRecog<Self> + 'static,
          TF: TokenFactory<'input>
{
    fn set_channel(&mut self, v: isize) {
        self.channel = v;
    }

    fn push_mode(&mut self, _m: usize) {
        self.mode_stack.push(self.mode);
        self.mode = _m;
    }

    fn pop_mode(&mut self) -> Option<usize> {
        self.mode_stack.pop().map(|mode| {
            self.mode = mode;
            mode
        })
    }

    fn set_type(&mut self, t: isize) {
        self.token_type = t;
    }

    fn set_mode(&mut self, m: usize) {
        self.mode = m;
    }

    fn more(&mut self) {
        self.set_type(LEXER_MORE)
    }

    fn skip(&mut self) {
        self.set_type(LEXER_SKIP)
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn get_interpreter(&self) -> Option<&LexerATNSimulator> { self.interpreter.as_ref() }
}
