
use crate::expression::Expression;
use crate::ui_helpers::split_expression;
use crate::ui_helpers::Selection;
use pancurses as pc;

struct WindowFmt<'a>(&'a pc::Window);
impl std::fmt::Write for WindowFmt<'_> {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		self.0.addstr(s);
		Ok( () )
	}
}

macro_rules! log {
	($win:expr, $($a:tt)*) => {{
		let win: &pc::Window = &$win;
		let p = win.get_cur_yx();
		win.mv( win.get_max_y() - 2, 0 );
		win.clrtoeol();
		use std::fmt::Write;
		let _ = write!(WindowFmt(win), $($a)*);
		win.mv(p.0, p.1);
		win.refresh();
		}};
}

struct Line {
	expr: Expression,
	sel: Selection,
}
impl Line {
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
	//fn shrink_right(&mut self) -> bool {
	//	self.sel.shrink_right(&self.expr)
	//}
	//fn shrink_left(&mut self) -> bool {
	//	self.sel.shrink_left(&self.expr)
	//}
}


pub fn mainloop()
{
	let e: Expression = "s = s_0 + u*t + 0.5*a*t^2".parse().unwrap();

	let window = pc::initscr();
	pc::noecho();
	pc::curs_set(0);	// Hide the cursor
  	window.keypad(true);

	let mut lines = vec![
		Line { expr: e, sel: Selection { path: vec![], first: 0, last: 0 }, },
		Line { expr: "v = v_0 + a*t".parse().unwrap(), sel: Selection { path: vec![], first: 0, last: 0 }, },
		];

	#[derive(PartialEq,Debug)]
	enum InputMode
	{
		LineSelect,
		ExprPick,	// Pick a single sub-expression
		ExprSelect,	// Select a range of sub-expressions
		ExprMove,	// Cursor keys move the insertion point around the same level as the selection, esc ends
		//Menu,	// Showing a menu
	}
	let mut cur_line = 0;
	let mut mode = InputMode::ExprPick;

	for (i, line) in lines.iter().enumerate()
	{
		window.mv(i as i32, 2);
		draw_expression_nosel(&window, &line.expr);
	}

	let mut last_line = 0;
	let mut redraw = true;
	loop {
		if cur_line != last_line {
			window.mv(last_line as i32, 2);
			draw_expression_nosel(&window, &lines[last_line].expr);
			last_line = cur_line;
			redraw = true;
		}
		if redraw {
			let line = &lines[cur_line];
			window.mv(cur_line as i32, 2);
			window.clrtoeol();
			if mode == InputMode::LineSelect { 
				window.attron(pc::Attribute::Bold);
				draw_expression_nosel(&window, &line.expr);
				window.attroff(pc::Attribute::Bold);
			}
			else {
				draw_expression(&window, &line.expr, &line.sel);
			}
			redraw = false;
		}
		window.mv(0, 0);
		window.refresh();

		match window.getch()
		{
		Some(pc::Input::Character('q')) => break,
		Some(pc::Input::Character('x')) => {
			// TODO: Allow moving the cursor to points within the current level
			},
		Some(pc::Input::Character('o')) => {
			// TODO: Show a list of allowed operations
			//let opid = show_modal_menu(&["Extract common factors", "Distribute", "Edit expression"]);
			},

		Some(pc::Input::KeyEnter) | Some(pc::Input::Character('\n')) =>
			match mode
			{
			InputMode::LineSelect => {
				mode = InputMode::ExprPick;
				redraw = true;
				},
			_ => {},
			},
		Some(pc::Input::Character('V')) => {
			mode = InputMode::LineSelect;
			redraw = true;
			},
		Some(pc::Input::Character('v')) => {
			mode = InputMode::ExprSelect;
			redraw = true;
			},
		Some(pc::Input::KeyUp) | Some(pc::Input::Character('k')) =>
			match mode
			{
			InputMode::LineSelect => {
				if cur_line > 0 {
					cur_line -= 1;
					redraw = true;
				}
				},
			InputMode::ExprPick => {
				if lines[cur_line].move_out() {
					log!(window, "Up pressed - move_out to {:?}", lines[cur_line].sel);
					redraw = true;
				}
				else {
					log!(window, "Up pressed - Can't ascend, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				// No up/down in select mode
				},
			InputMode::ExprMove => {
				// No up/down in move mode
				},
			},
		Some(pc::Input::KeyDown) | Some(pc::Input::Character('j')) =>
			match mode
			{
			InputMode::LineSelect => {
				if cur_line+1 < lines.len() {
					cur_line += 1;
					redraw = true;
				}
				},
			InputMode::ExprPick => {
				if lines[cur_line].move_in() {
					log!(window, "Down pressed - move_in to {:?}", lines[cur_line].sel);
					redraw = true;
				}
				else {
					log!(window, "Down pressed - Can't decend, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				// No up/down in select mode
				},
			InputMode::ExprMove => {
				// No up/down in move mode
				},
			},
		Some(pc::Input::KeyRight) | Some(pc::Input::Character('l')) => {
			if lines[cur_line].shift_right() {
				log!(window, "Right pressed - shift_right to {:?}", lines[cur_line].sel);
				redraw = true;
			}
			else {
				log!(window, "Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
			}
			},
		Some(pc::Input::KeyLeft) | Some(pc::Input::Character('h')) => {
			if lines[cur_line].shift_left() {
				log!(window, "Left pressed - shift_right to {:?}", lines[cur_line].sel);
				redraw = true;
			}
			else {
				log!(window, "Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
			}
			},
		Some(pc::Input::KeySRight) | Some(pc::Input::Character('L')) => {
			if lines[cur_line].expand_right() {
				log!(window, "Alt Right pressed - expand_right to {:?}", lines[cur_line].sel);
				redraw = true;
			}
			else {
				log!(window, "Alt Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
			}
			},
		Some(pc::Input::KeySLeft) | Some(pc::Input::Character('H')) => {
			if lines[cur_line].expand_left() {
				log!(window, "Alt Left pressed - shift_right to {:?}", lines[cur_line].sel);
				redraw = true;
			}
			else {
				log!(window, "Alt Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
			}
			},
		k @ _ => {
			log!(window, "Unknown key {:?}", k);
			},
		}
	}
	pc::endwin();
}

fn draw_expression_nosel(win: &pc::Window, e: &Expression)
{
	use std::fmt::Write;
	write!(WindowFmt(win), "{}", e);
}
fn draw_expression(win: &pc::Window, e: &Expression, sel: &Selection)
{
	let (before, hilight, after,) = split_expression(e, sel);
	if hilight.len() > 0
	{
		win.addstr(&before);
		win.attron(pc::Attribute::Underline);
		win.addstr(&hilight);
		win.attroff(pc::Attribute::Underline);
		win.addstr(&after);
	}
	else
	{
		assert!(after.len() == 0);
		win.addstr(&before);
	}
}

