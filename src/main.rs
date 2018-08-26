
#[macro_use]
extern crate structopt;

mod expression;
mod curses_ui;
mod ui_helpers;
mod manip;

#[derive(StructOpt)]
#[structopt(name="equation", about="Algebraic equation editor")]
struct Opts
{
	#[structopt(parse(from_os_str))]
	infile: Option<std::path::PathBuf>,
	#[structopt(short="R", long="read-only")]
	readonly: bool,
}

fn main()
{
	let opts: Opts = structopt::StructOpt::from_args();

	let mut lines = if let Some(ref v) = opts.infile
		{
			EquationSet::from_file(v).unwrap().lines
		}
		else
		{
			vec![
				Line::from_str("s = s_0 + u*t + 0.5*a_0*t^2 + 1/6*j*t^3"),
				Line::from_str("v = v_0 + a_0*t + 0.5*j*t^2"),
				Line::from_str("a = a_0 + j*t"),
				]
		};

	curses_ui::mainloop(&mut lines);
}

pub struct EquationSet {
	pub dirty: bool,
	pub lines: Vec<Line>,
}
impl EquationSet
{
	pub fn from_file(p: &std::path::Path) -> std::io::Result<EquationSet>
	{
		use std::io::BufRead;
		let f = std::io::BufReader::new( std::fs::File::open(p)? );

		let mut rv = EquationSet {
			dirty: false,
			lines: Vec::new(),
			};
		for line in f.lines()
		{
			rv.lines.push(Line::from_str(&line?));
		}
		Ok( rv )
	}

	pub fn save_to(&self, p: &std::path::Path) -> std::io::Result<()>
	{
		use std::io::Write;
		let mut f = std::io::BufWriter::new( std::fs::File::create(p)? );

		for l in self.lines.iter()
		{
			write!(f, "{}", l.expr)?;
			if l.comment.len() > 0 {
				write!(f, " #{}", l.comment)?;
			}
			write!(f, "\n")?;
		}

		Ok( () )
	}
}


#[derive(Clone)]
pub struct Line {
	expr: expression::Expression,
	comment: String, 
	sel: ui_helpers::Selection,
}
impl Line {
	fn from_expr(expr: expression::Expression) -> Line {
		Line {
			expr: expr,
			comment: "".to_owned(), 
			sel: crate::ui_helpers::Selection { path: vec![], first: 0, last: 0 },
		}
	}
	fn from_str(s: &str) -> Line {
		let (expr, comment) = expression::Expression::parse_from_str_with_comment(s).unwrap();
		Line {
			expr: expr,
			comment: comment,
			sel: crate::ui_helpers::Selection { path: vec![], first: 0, last: 0 },
		}
	}

	fn render_split(&self) -> (String,String,String) {
		crate::ui_helpers::split_expression(&self.expr, &self.sel)
	}
	fn render_selection(&self) -> String {
		crate::ui_helpers::split_expression(&self.expr, &self.sel).1
	}

	fn extract_selection(&self) -> expression::Expression {
		crate::ui_helpers::extract_subexpression(&self.expr, &self.sel)
	}
	fn replace_selection(&mut self, e: expression::Expression) {
		crate::ui_helpers::replace_subexpression(&mut self.expr, &mut self.sel, e)
	}

	fn move_out(&mut self) -> bool {
		self.sel.move_out(&self.expr)
	}
	fn move_in(&mut self) -> bool {
		self.sel.move_in(&self.expr)
	}
	fn shift_right(&mut self) -> bool {
		self.sel.shift_right(&self.expr)
	}
	fn shift_left(&mut self) -> bool {
		self.sel.shift_left(&self.expr)
	}
	fn expand_right(&mut self) -> bool {
		self.sel.expand_right(&self.expr)
	}
	fn expand_left(&mut self) -> bool {
		self.sel.expand_left(&self.expr)
	}
	fn shrink_right(&mut self) -> bool {
		self.sel.shrink_right(&self.expr)
	}
	fn shrink_left(&mut self) -> bool {
		self.sel.shrink_left(&self.expr)
	}
}

