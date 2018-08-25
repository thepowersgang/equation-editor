
#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum Op
{
	Equality,
	Add,	// Note: Subtract is handled with `negated`
	Multiply,
	Divide,
	Exponent,
}
#[derive(Debug,Clone)]
pub enum Expression
{
	SubNode(ExprNode),
	Literal(f32),
	Variable(String),
}
#[derive(Debug,Clone)]
pub struct SubExpression
{
	pub negated: bool,
	pub val: Expression,
}

/// Representation of a chained set of expressions (e.g. `a + b + c` or `a / b / c`)
#[derive(Debug,Clone)]
pub struct ExprNode
{
	pub operation: Op,
	pub values: Vec<SubExpression>,
}

#[derive(PartialEq,PartialOrd,Eq,Ord)]
enum Precedence
{
	Equality,
	AddSub,
	Div,
	Mul,
	Exp,
	Lit,
}
impl Precedence
{
	fn of_op(op: Op) -> Precedence
	{
		match op
		{
		Op::Equality => Precedence::Equality,
		Op::Add => Precedence::AddSub,
		Op::Multiply => Precedence::Mul,
		Op::Divide => Precedence::Div,
		Op::Exponent => Precedence::Exp,
		}
	}
	fn of_expr(e: &Expression) -> Precedence
	{
		match e
		{
		Expression::SubNode(sn) => Precedence::of_op(sn.operation),
		_ => Precedence::Lit,
		}
	}
}
impl Expression {
	pub fn needs_parens(&self, op: Op) -> bool {
		Precedence::of_expr(self) <= Precedence::of_op(op)
	}
}


impl std::str::FromStr for Expression
{
	type Err = String;
	fn from_str(s: &str) -> Result<Expression, String> {
		let mut l = Lexer::new(s);
		Self::parse_root(&mut l).map_err(|e| format!("{:?}", e))
	}
}
impl std::fmt::Display for Expression
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self
		{
		Expression::SubNode(sn) => std::fmt::Display::fmt(sn, f),
		Expression::Literal(v) => std::fmt::Display::fmt(&v, f),
		Expression::Variable(n) => std::fmt::Display::fmt(&n[..], f),
		}
	}
}
impl std::fmt::Display for ExprNode
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		use std::fmt::Write;
		assert!(self.values.len() > 1);
		let mut it = self.values.iter();

		let emit_with_parens = |v: &Expression, f: &mut std::fmt::Formatter| {
			let needs_parens = v.needs_parens(self.operation);
			if needs_parens {
				f.write_char('(')?;
			}
			std::fmt::Display::fmt(v, f)?;
			if needs_parens {
				f.write_char(')')?;
			}
			Ok( () )
			};

		{
			let v0 = it.next().unwrap();
			if v0.negated {
				f.write_char('-')?;
			}
			emit_with_parens(&v0.val, f)?;
		}
		for v in it
		{
			match self.operation
			{
			Op::Add => if v.negated {
					f.write_char('-')?;
				}
				else {
					f.write_char('+')?;
				},
			Op::Multiply => f.write_char('*')?,
			Op::Divide => f.write_char('/')?,
			Op::Exponent => f.write_char('^')?,
			Op::Equality => f.write_char('=')?,
			}
			if v.negated && self.operation != Op::Add {
				f.write_char('-')?;
			}
			emit_with_parens(&v.val, f)?;
		}
		Ok( () )
	}
}

#[derive(Debug)]
enum ParseError {
	Unexpected(String),
}
#[derive(Debug,Copy,Clone,PartialEq)]
enum Token<'a> {
	Eof,
	Whitespace,
	Ident(&'a str),
	Literal(f32),
	Op(char),
	ParenOpen,
	ParenClose,
}
::plex::lexer! {
	fn lex_next_token(text: 'a) -> (Token<'a>, &'a str);

	r#"[ \t\r\n]+"# => (Token::Whitespace, text),
	r#"[0-9]+(\.[0-9]*)?"# => (
            if let Ok(i) = text.parse() {
                Token::Literal(i)
            } else {
                panic!("integer {} is out of range", text)
            }, text),
	r#"[a-zA-Z][a-zA-Z0-9_']*"# => (Token::Ident(text), text),
	r#"\+"# => (Token::Op('+'), text),
	r#"-"#  => (Token::Op('-'), text),
	r#"\*"# => (Token::Op('*'), text),
	r#"/"#  => (Token::Op('/'), text),
	r#"\^"# => (Token::Op('^'), text),
	r#"="#  => (Token::Op('='), text),

	r#"\("# => (Token::ParenOpen, text),
	r#"\)"# => (Token::ParenClose, text),
	r"." => panic!("Unexpected character: {}", text),
}
struct Lexer<'a>
{
	//base: &'a str,
	remaining: &'a str,
	cur_token: Token<'a>,
}
impl<'a> Lexer<'a>
{
	fn new(s: &'a str) -> Lexer {
		let mut rv = Lexer {
			//base: s,
			remaining: s,
			cur_token: Token::Eof,
			};
		rv.consume();
		rv
	}
	pub fn consume(&mut self) -> Token<'a> {
		let mut t;
		loop
		{
			t = if let Some( ( (tok,_), new_rem) ) = lex_next_token(self.remaining) {
					self.remaining = new_rem;
					tok
				}
				else {
					Token::Eof
				};
			if t != Token::Whitespace {
				break;
			}
		}
		//println!("{:?} => {:?}", self.cur_token, t);
		::std::mem::replace(&mut self.cur_token, t)
	}
	pub fn cur(&self) -> Token<'a> {
		self.cur_token
	}
	pub fn consume_if(&mut self, t: Token<'_>) -> bool {
		if self.cur_token == t {
			self.consume();
			true
		}
		else {
			false
		}
	}
}

impl Expression
{
	fn parse_root(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		Self::parse_0(lexer)
	}
	fn parse_0(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		let v = Self::parse_1(lexer)?;
	
		if let Token::Op('=') = lexer.cur() {
			let mut values = vec![v];
			while lexer.consume_if( Token::Op('=') ) {
				values.push( Self::parse_1(lexer)? );
			}
			Ok(Expression::SubNode(ExprNode {
				operation: Op::Equality,
				values: values,
				}))
		}
		else {
			assert!( !v.negated );
			Ok(v.val)
		}
	}
	// Add/Subtract
	fn parse_1(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		let mut v = Self::parse_2(lexer)?;
		let mut values = vec![];

		loop
		{
			let is_neg = if lexer.consume_if(Token::Op('-')) {
					true
				}
				else if lexer.consume_if(Token::Op('+')) {
					false
				}
				else {
					break;
				};
			values.push(v);
			v = Self::parse_2(lexer)?;
			v.negated ^= is_neg;
		}
		if values.len() > 0
		{
			values.push(v);
			Ok(SubExpression { negated: false, val: Expression::SubNode(ExprNode {
				operation: Op::Add,
				values: values,
				}) })
		}
		else
		{
			Ok(v)
		}
	}
	// Multiply
	fn parse_2(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		let mut v = Self::parse_3(lexer)?;
		let mut values = vec![];
		while lexer.consume_if( Token::Op('*') ) {
			values.push(v);
			v = Self::parse_3(lexer)?;
		}
		if values.len() > 0
		{
			values.push(v);
			Ok(SubExpression { negated: false, val: Expression::SubNode(ExprNode {
				operation: Op::Multiply,
				values: values,
				}) })
		}
		else
		{
			Ok(v)
		}
	}
	// Divide
	fn parse_3(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		let mut v = Self::parse_4(lexer)?;
		let mut values = vec![];
		while lexer.consume_if( Token::Op('/') ) {
			values.push(v);
			v = Self::parse_4(lexer)?;
		}
		if values.len() > 0
		{
			values.push(v);
			Ok(SubExpression { negated: false, val: Expression::SubNode(ExprNode {
				operation: Op::Divide,
				values: values,
				}) })
		}
		else
		{
			Ok(v)
		}
	}
	// Negation
	fn parse_4(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		let flip_sign = lexer.consume_if(Token::Op('-'));
		let mut v = Self::parse_5(lexer)?;
		v.negated ^= flip_sign;
		Ok(v)
	}
	// Exponent
	fn parse_5(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		let v = Self::parse_6(lexer)?;
		if let Token::Op('^') = lexer.cur() {
			let mut values = vec![v];
			while lexer.consume_if( Token::Op('^') ) {
				// TODO: The exponent can be negative
				values.push( Self::parse_6(lexer)? );
			}
			Ok(SubExpression { negated: false, val: Expression::SubNode(ExprNode {
				operation: Op::Exponent,
				values: values,
				}) })
		}
		else {
			Ok(v)
		}
	}
	fn parse_6(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		Self::parse_value(lexer)
	}
	fn parse_value(lexer: &mut Lexer) -> Result<SubExpression,ParseError> {
		Ok(SubExpression { negated: false, val: match lexer.cur()
			{
			Token::Literal(v) => {
				lexer.consume();
				Expression::Literal(v)
				},
			Token::Ident(i) => {
				lexer.consume();
				Expression::Variable(i.to_owned())
				},
			Token::ParenOpen => {
				lexer.consume();
				let rv = Self::parse_1(lexer)?;
				if !lexer.consume_if(Token::ParenClose) {
					return Err(ParseError::Unexpected( format!("{:?}", lexer.cur()) ));
				}
				return Ok(rv)
				},
			_ => return Err(ParseError::Unexpected( format!("{:?}", lexer.cur()) )),
			} })
	}
}


