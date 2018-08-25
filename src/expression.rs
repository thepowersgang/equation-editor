
//!
//! Expression type
//!

#[derive(Debug,PartialEq,Eq,Copy,Clone)]
pub enum Op
{
	Equality,
	AddSub,	// Note: Subtract is handled with `negated`
	MulDiv,
	ExpRoot,	// NOTE: Root doesn't actually exist
}
#[derive(Debug,Clone)]
pub enum Expression
{
	/// Negate the inner value
	Negative(Box<Expression>),
	/// Group of like operators
	SubNode(ExprNode),
	/// Literal value
	Literal(f32),
	/// Variable name
	Variable(String),
}
#[derive(Debug,Clone)]
pub struct SubExpression
{
	/// Indicates subtract/divide instead of add/multiply
	pub inverse: bool,
	pub val: Expression,
}

/// Representation of a chained set of expressions with the same precedence (e.g. `a + b - c` or `a / b * c`)
#[derive(Debug,Clone)]
pub struct ExprNode
{
	pub operation: Op,
	pub values: Vec<SubExpression>,
}
#[derive(Debug)]
pub enum ParseError {
	Unexpected(String),
	BadToken(String),
}

#[derive(PartialEq,PartialOrd,Eq,Ord)]
enum Precedence
{
	Equality,
	AddSub,
	MulDiv,
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
		Op::AddSub => Precedence::AddSub,
		Op::MulDiv => Precedence::MulDiv,
		Op::ExpRoot => Precedence::Exp,
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
	type Err = ParseError;
	fn from_str(s: &str) -> Result<Expression, ParseError> {
		let mut l = Lexer::new(s)?;
		Self::parse_root(&mut l)
	}
}
impl std::fmt::Display for Expression
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self
		{
		Expression::Negative(sn) =>
			match **sn
			{
			Expression::Literal(_) => write!(f, "-{}", sn),
			Expression::Variable(_) => write!(f, "-{}", sn),
			_ => write!(f, "-({})", sn),
			},
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

		for (i,v) in self.values.iter().enumerate()
		{
			if i == 0 {
			}
			else {
				match self.operation
				{
				Op::AddSub => if v.inverse {
						f.write_char('-')?;
					}
					else {
						f.write_char('+')?;
					},
				Op::MulDiv => if v.inverse {
						f.write_char('/')?;
					}
					else {
						f.write_char('*')?;
					},
				Op::ExpRoot => { assert!(!v.inverse); f.write_char('^')? },
				Op::Equality => f.write_char('=')?,
				}
			}
			emit_with_parens(&v.val, f)?;
		}
		Ok( () )
	}
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum Token<'a> {
	Eof,
	Whitespace,
	Comment(&'a str),
	Ident(&'a str),
	Literal(f32),
	Op(char),
	ParenOpen,
	ParenClose,
}
::plex::lexer! {
	fn lex_next_token(text: 'a) -> Result<Token<'a>, ParseError>;

	r#"[ \t\r\n]+"# => Ok(Token::Whitespace),
	r#"#.*"# => Ok(Token::Comment(text)),
	r#"[0-9]+(\.[0-9]*)?"# =>
            if let Ok(i) = text.parse() {
                Ok(Token::Literal(i))
            } else {
				Err(ParseError::BadToken(text.to_owned()))
            },
	r#"[a-zA-Z][a-zA-Z0-9_']*"# => Ok(Token::Ident(text)),
	r#"\+"# => Ok(Token::Op('+')),
	r#"-"#  => Ok(Token::Op('-')),
	r#"\*"# => Ok(Token::Op('*')),
	r#"/"#  => Ok(Token::Op('/')),
	r#"\^"# => Ok(Token::Op('^')),
	r#"="#  => Ok(Token::Op('=')),

	r#"\("# => Ok(Token::ParenOpen),
	r#"\)"# => Ok(Token::ParenClose),
	r"." => Err(ParseError::BadToken(text.to_owned())),
}
struct Lexer<'a>
{
	//base: &'a str,
	remaining: &'a str,
	cur_token: Token<'a>,
}
impl<'a> Lexer<'a>
{
	fn new(s: &'a str) -> Result<Lexer<'a>, ParseError> {
		let mut rv = Lexer {
			//base: s,
			remaining: s,
			cur_token: Token::Eof,
			};
		rv.consume()?;
		Ok(rv)
	}
	pub fn consume(&mut self) -> Result<Token<'a>, ParseError> {
		let mut t;
		loop
		{
			t = if let Some((tok_res, new_rem)) = lex_next_token(self.remaining) {
					self.remaining = new_rem;
					tok_res?
				}
				else {
					Token::Eof
				};
			if let Token::Comment(_) = t {
				continue ;
			}
			if t == Token::Whitespace {
				continue ;
			}
			break;
		}
		Ok( ::std::mem::replace(&mut self.cur_token, t) )
	}
	pub fn cur(&self) -> Token<'a> {
		self.cur_token
	}
	pub fn consume_if(&mut self, t: Token<'_>) -> Result<bool,ParseError> {
		Ok(if self.cur_token == t {
			self.consume()?;
			true
		}
		else {
			false
		})
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
			let mut values = vec![SubExpression { inverse: false, val: v }];
			while lexer.consume_if( Token::Op('=') )? {
				values.push(SubExpression { inverse: false, val: Self::parse_1(lexer)? });
			}
			Ok(Expression::SubNode(ExprNode {
				operation: Op::Equality,
				values: values,
				}))
		}
		else {
			Ok(v)
		}
	}
	// Add/Subtract
	fn parse_1(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		let mut v = Self::parse_2(lexer)?;
		let mut values = vec![];
		let mut is_neg = false;

		loop
		{
			let new_is_neg = if lexer.consume_if(Token::Op('-'))? {
					true
				}
				else if lexer.consume_if(Token::Op('+'))? {
					false
				}
				else {
					break;
				};
			values.push(SubExpression { inverse: is_neg, val: v });
			is_neg = new_is_neg;
			v = Self::parse_2(lexer)?;
		}
		if values.len() > 0
		{
			values.push(SubExpression { inverse: is_neg, val: v });
			Ok( Expression::SubNode(ExprNode {
				operation: Op::AddSub,
				values: values,
				}) )
		}
		else
		{
			Ok(v)
		}
	}
	// Multiply / Divide
	fn parse_2(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		let mut v = Self::parse_3(lexer)?;
		let mut values = vec![];
		let mut is_div = false;

		loop
		{
			let new_is_div = if lexer.consume_if(Token::Op('/'))? {
					true
				}
				else if lexer.consume_if(Token::Op('*'))? {
					false
				}
				else {
					break;
				};
			values.push(SubExpression { inverse: is_div, val: v });
			is_div = new_is_div;
			v = Self::parse_3(lexer)?;
		}
		if values.len() > 0
		{
			values.push(SubExpression { inverse: is_div, val: v });
			Ok( Expression::SubNode(ExprNode {
				operation: Op::MulDiv,
				values: values,
				}) )
		}
		else
		{
			Ok(v)
		}
	}
	// Unary Negation
	fn parse_3(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		if lexer.consume_if(Token::Op('-'))?
		{
			let v = Self::parse_4(lexer)?;
			Ok(Expression::Negative( Box::new(v) ))
		}
		else
		{
			Self::parse_4(lexer)
		}
	}
	// Exponent
	fn parse_4(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		let mut v = Self::parse_5(lexer)?;
		let mut values = vec![];

		loop
		{
			if lexer.consume_if(Token::Op('^'))? {
				false
			}
			else {
				break;
			};
			values.push(SubExpression { inverse: false, val: v });
			v = Self::parse_5(lexer)?;
		}
		if values.len() > 0
		{
			values.push(SubExpression { inverse: false, val: v });
			Ok( Expression::SubNode(ExprNode {
				operation: Op::ExpRoot,
				values: values,
				}) )
		}
		else
		{
			Ok(v)
		}
	}
	fn parse_5(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		Self::parse_value(lexer)
	}
	fn parse_value(lexer: &mut Lexer) -> Result<Expression,ParseError> {
		Ok(match lexer.cur()
			{
			Token::Literal(v) => {
				lexer.consume()?;
				Expression::Literal(v)
				},
			Token::Ident(i) => {
				lexer.consume()?;
				Expression::Variable(i.to_owned())
				},
			Token::ParenOpen => {
				lexer.consume()?;
				let rv = Self::parse_1(lexer)?;
				if !lexer.consume_if(Token::ParenClose)? {
					return Err(ParseError::Unexpected( format!("{:?}", lexer.cur()) ));
				}
				return Ok(rv)
				},
			_ => return Err(ParseError::Unexpected( format!("{:?}", lexer.cur()) )),
			})
	}
}


