
mod expression;
mod curses_ui;

mod ui_helpers;

fn main()
{
	let mut lines = vec![
		Line::from_str("s = s_0 + u*t + 0.5*a_0*t^2 + 1/6*j*t^3"),
		Line::from_str("v = v_0 + a_0*t + 0.5*j*t^2"),
		Line::from_str("a = a_0 + j*t"),
		];
	curses_ui::mainloop(&mut lines);
}


#[derive(Clone)]
pub struct Line {
	expr: expression::Expression,
	sel: ui_helpers::Selection,
}
impl Line {
	fn from_str(s: &str) -> Line {
		Line {
			expr: s.parse().unwrap(),
			sel: crate::ui_helpers::Selection { path: vec![], first: 0, last: 0 },
		}
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

