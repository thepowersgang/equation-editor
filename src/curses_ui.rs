//!
//! pancurses backed UI
//!

use crate::expression::Expression;
//use crate::ui_helpers::Selection;
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

pub fn mainloop(lines: &mut Vec<super::Line>)
{
	let window = pc::initscr();
	pc::noecho();
	pc::curs_set(0);	// Hide the cursor
  	window.keypad(true);

	#[derive(PartialEq,Debug)]
	enum InputMode
	{
		LineSelect,
		ExprPick,	// Pick a single sub-expression
		ExprSelect,	// Select a range of sub-expressions
		ExprMove,	// Cursor keys move the insertion point around the same level as the selection, esc ends
	}
	#[derive(PartialEq,Debug)]
	enum Redraw
	{
		None,
		Current,
		All,
	}
	enum Clipboard
	{
		Empty,
		Line(crate::Line),
		Expr(Expression),
	}
	let mut cur_line = 0;
	let mut mode = InputMode::LineSelect;
	let mut clipboard = Clipboard::Empty;
	let mut statusline = std::borrow::Cow::from("");

	let mut last_line = 0;
	let mut redraw = Redraw::All;
	loop
	{
		// Re-draw the entire set of expressions
		if redraw == Redraw::All
		{
			for y in 0 .. window.get_max_y() {
				if y == window.get_max_y() - 2 {
					// Skip the debug line
					continue ;
				}
				window.mv(y, 0);
				window.hline(' ', window.get_max_x());
			}
			for (i, line) in lines.iter().enumerate()
			{
				window.mv(i as i32, 2);
				draw_expression_nosel(&window, &line.expr);
			}
		}

		// If the current line changed, re-render the old line with no selection
		if cur_line != last_line
		{
			if redraw != Redraw::All
			{
				window.mv(last_line as i32, 2);
				draw_expression_nosel(&window, &lines[last_line].expr);
			}

			//cur_sel = Selection::new();
			last_line = cur_line;
			redraw = Redraw::Current;
		}

		if redraw != Redraw::None
		{
			let line = &lines[cur_line];
			window.mv(cur_line as i32, 2);
			window.clrtoeol();
			if mode == InputMode::LineSelect { 
				window.attron(pc::Attribute::Bold);
				draw_expression_nosel(&window, &line.expr);
				window.attroff(pc::Attribute::Bold);
			}
			else {
				draw_expression(&window, &line);
			}
			
			{
				window.mv( window.get_max_y() - 1, 0 );
				window.addstr(&statusline);
				statusline = "".into();
				let v = match mode
					{
					InputMode::LineSelect => "LINE",
					InputMode::ExprPick => "PICK",
					InputMode::ExprSelect => "SELECT",
					InputMode::ExprMove => "MOVE",
					};
				window.mv( window.get_max_y() - 1, window.get_max_x() - 10 );
				window.addstr(v);
			}
		}
		redraw = Redraw::None;
		// TODO: If in InputMode ExprSelect, move the cursor to the RHS (or controlled side) of the selection?
		window.mv(0, 0);
		window.refresh();

		match window.getch()
		{
		Some(pc::Input::Character('q')) => break,
		Some(pc::Input::Character('.')) => {
			if let Some(opid) = show_menu_modal(&window, &["Extract common factors", "Distribute"])
			{
				match opid
				{
				0 => log!(window, "TODO: Extract common factors"),
				1 => log!(window, "TODO: Distribute leading multiplication"),
				_ => log!(window, "BUG: Unknown operation {}", opid),
				}
			}
			else
			{
				log!(window, "No command selected");
			}
			redraw = Redraw::All;
			},

		Some(pc::Input::KeyEnter) | Some(pc::Input::Character('\n')) =>
			match mode
			{
			InputMode::LineSelect => {
				mode = InputMode::ExprPick;
				redraw = Redraw::Current;
				},
			InputMode::ExprSelect | InputMode::ExprPick => {
				mode = InputMode::ExprMove;
				},
			_ => {},
			},

		Some(pc::Input::Character('a')) | Some(pc::Input::Character('i')) | Some(pc::Input::Character('e')) =>
			match mode
			{
			InputMode::LineSelect => {
				let s = format!("{}", lines[cur_line].expr);
				let v = show_input_modal(&window, &s);
				match v.parse::<crate::expression::Expression>()
				{
				Ok(expr) => {
					lines[cur_line].expr = expr;
					},
				Err(e) => {
					statusline = format!("Error parsing: {:?}", e).into();
					},
				}
				redraw = Redraw::All;
				},
			InputMode::ExprSelect | InputMode::ExprPick => {
				let s = lines[cur_line].render_selection();
				let v = show_input_modal(&window, &s);
				match v.parse::<crate::expression::Expression>()
				{
				Ok(expr) => {
					lines[cur_line].replace_selection( expr );
					},
				Err(e) => {
					statusline = format!("Error parsing: {:?}", e).into();
					},
				}
				redraw = Redraw::All;
				},
			_ => {},
			}
		Some(pc::Input::Character('o')) =>
			match mode
			{
			InputMode::LineSelect => {
				let v = show_input_modal(&window, "");
				match v.parse::<crate::expression::Expression>()
				{
				Ok(expr) => {
					lines.insert(cur_line + 1, crate::Line::from_expr(expr));
					},
				Err(e) => {
					statusline = format!("Error parsing: {:?}", e).into();
					},
				}
				redraw = Redraw::All;
				},
			_ => {},
			},
		Some(pc::Input::Character('O')) =>
			match mode
			{
			InputMode::LineSelect => {
				let v = show_input_modal(&window, "");
				match v.parse::<crate::expression::Expression>()
				{
				Ok(expr) => {
					lines.insert(cur_line, crate::Line::from_expr(expr));
					},
				Err(e) => {
					statusline = format!("Error parsing: {:?}", e).into();
					},
				}
				redraw = Redraw::All;
				},
			_ => {},
			},

		Some(pc::Input::Character('D')) =>
			match mode
			{
			InputMode::LineSelect => {
				if cur_line < lines.len() {
					clipboard = Clipboard::Line( lines.remove(cur_line) );
					// TODO: Avoid this?
					if cur_line != 0 {
						cur_line -= 1;
					}
					statusline = "Line moved to clipboard".into();
				}
				else {
					// TODO: Warning?
				}
				redraw = Redraw::All;
				},
			InputMode::ExprSelect | InputMode::ExprPick => {
				// TODO: Cut/Paste expressions
				// - Reqires removing the expression
				},
			_ => {},
			},
		Some(pc::Input::Character('Y')) | Some(pc::Input::Character('y')) =>
			match mode
			{
			InputMode::LineSelect => {
				if cur_line < lines.len() {
					clipboard = Clipboard::Line( lines[cur_line].clone() );
					statusline = "Line copied to clipboard".into();
					redraw = Redraw::Current;
				}
				else {
					// TODO: Warning?
				}
				},
			InputMode::ExprSelect | InputMode::ExprPick => {
				clipboard = Clipboard::Expr( lines[cur_line].extract_selection() );
				statusline = "Expression copied to clipboard".into();
				redraw = Redraw::Current;
				},
			_ => {},
			},
		Some(pc::Input::Character('P')) =>
			match mode
			{
			InputMode::LineSelect =>
				match std::mem::replace(&mut clipboard, Clipboard::Empty)
				{
				Clipboard::Empty => {},
				Clipboard::Expr(_) => {},
				Clipboard::Line(l) => {
					lines.insert(cur_line, l);
					redraw = Redraw::All;
					},
				},
			_ => {},
			},
		Some(pc::Input::Character('p')) =>
			match mode
			{
			InputMode::LineSelect =>
				match std::mem::replace(&mut clipboard, Clipboard::Empty)
				{
				Clipboard::Empty => {},
				Clipboard::Expr(_) => {},
				Clipboard::Line(l) => {
					if cur_line < lines.len() {
						cur_line += 1;
					}
					lines.insert(cur_line, l);
					redraw = Redraw::All;
					},
				},
			_ => {},
			},

		Some(pc::Input::Character('V')) => {
			mode = InputMode::LineSelect;
			redraw = Redraw::Current;
			},
		Some(pc::Input::Character('v')) => {
			mode = InputMode::ExprSelect;
			redraw = Redraw::Current;
			},
		Some(pc::Input::KeyUp) | Some(pc::Input::Character('k')) =>
			match mode
			{
			InputMode::LineSelect => {
				if cur_line > 0 {
					cur_line -= 1;
					redraw = Redraw::Current;
				}
				},
			InputMode::ExprPick => {
				if lines[cur_line].move_out() {
					log!(window, "Up pressed - move_out to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
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
					redraw = Redraw::Current;
				}
				},
			InputMode::ExprPick => {
				if lines[cur_line].move_in() {
					log!(window, "Down pressed - move_in to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
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
		Some(pc::Input::KeyRight) | Some(pc::Input::Character('l')) => 
			match mode
			{
			InputMode::LineSelect => {
				// No left/right in line select mode
				},
			InputMode::ExprPick => {
				if lines[cur_line].shift_right() {
					log!(window, "Right pressed - shift_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				if lines[cur_line].expand_right() {
					log!(window, "Alt Right pressed - expand_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Alt Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprMove => {
				// Move the cursor
				},
			},
		Some(pc::Input::KeyLeft) | Some(pc::Input::Character('h')) => 
			match mode
			{
			InputMode::LineSelect => {
				// No left/right in line select mode
				},
			InputMode::ExprPick => {
				if lines[cur_line].shift_left() {
					log!(window, "Left pressed - shift_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				if lines[cur_line].shrink_right() {
					log!(window, "Left pressed - expand_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprMove => {
				// Move the insertion cursor
				},
			},
		Some(pc::Input::KeySRight) | Some(pc::Input::Character('L')) =>
			match mode
			{
			InputMode::LineSelect => {
				// No left/right in line select mode
				},
			InputMode::ExprPick => {
				if lines[cur_line].expand_right() {
					log!(window, "Alt Right pressed - expand_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Alt Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				if lines[cur_line].shrink_left() {
					log!(window, "Alt Right pressed - expand_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Alt Right pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprMove => {
				// No action
				},
			},
		Some(pc::Input::KeySLeft) | Some(pc::Input::Character('H')) =>
			match mode
			{
			InputMode::LineSelect => {
				// No left/right in line select mode
				},
			InputMode::ExprPick => {
				if lines[cur_line].expand_left() {
					log!(window, "Alt Left pressed - shift_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Alt Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprSelect => {
				if lines[cur_line].expand_left() {
					log!(window, "Shift Left pressed - shift_right to {:?}", lines[cur_line].sel);
					redraw = Redraw::Current;
				}
				else {
					log!(window, "Shift Left pressed - Can't move, staying at {:?}", lines[cur_line].sel);
				}
				},
			InputMode::ExprMove => {
				// No action
				},
			},
		k @ _ => {
			log!(window, "Unknown key {:?}", k);
			},
		}
	}
	pc::endwin();
}

fn show_input_modal(win: &pc::Window, prime_value: &str) -> String
{
	let (mut before, mut after) = (prime_value.to_owned(), Vec::<char>::new(),);

	let w = win.get_max_x() - 4;
	let h = 3;
	let x = 1;
	let y = win.get_max_y() / 2 - h/2;
	win.mv(y+0,x);
	win.addch('/');
	for _ in 0 .. w {
		win.addch('-');
	}
	win.addch('\\');
	win.mv(y+1,x);
	win.addch('|');
	for _ in 0 .. w {
		win.addch(' ');
	}
	win.addch('|');
	win.mv(y+2,x);
	win.addch('\\');
	for _ in 0 .. w {
		win.addch('-');
	}
	win.addch('/');

	loop
	{
		if true
		{
			win.mv(y+1,x+2);
			win.addstr(&before);
			for v in after.iter().rev() {
				win.addch(*v);
			}
			for _ in (before.len() + after.len()) .. w as usize {
				win.addch(' ');
			}

			win.mv(y+1, x+2+before.len() as i32);
			pc::curs_set(1);	// Hide the cursor
			win.refresh();
		}

		match win.getch()
		{
		Some(pc::Input::KeyEnter) | Some(pc::Input::Character('\n')) => {
			pc::curs_set(0);	// Hide the cursor
			while let Some(v) = after.pop() {
				before.push(v);
			}
			return before;
			},
		Some(pc::Input::KeyRight) => {
			if let Some(c) = after.pop() {
				before.push(c)
			}
			},
		Some(pc::Input::KeyLeft) => {
			if let Some(c) = before.pop() {
				after.push(c)
			}
			},
		Some(pc::Input::KeyBackspace) => {
			let _ = before.pop();
			},
		Some(pc::Input::Character(v @ ' ' ... 'z')) => {
			before.push(v);
			},
		_ => {},
		}

	}
}
fn show_menu_modal(win: &pc::Window, options: &[&str]) -> Option<usize>
{
	let h = options.len() as i32;
	let w = std::cmp::max( options.iter().map(|v| v.len()).max().unwrap_or(0), "Cancel".len() ) as i32;

	let x = win.get_max_x() / 2 - (w+1) / 2 - 2;
	let y = win.get_max_y() / 2 - (h+1) / 2 - 1;

	let mut cur_sel = options.len();
	loop
	{
		if true
		{
			win.mv(y,x);
			win.addch('/');
			win.addch('-');
			win.hline('-', w);
			win.mv(y,x+w+2);
			win.addch('-');
			win.addch('\\');
			for (i,o) in Iterator::chain(options.iter(), ["Cancel"].iter()).enumerate()
			{
				win.mv(y + 1 + i as i32, x);
				win.addch('|');
				win.addch(if i == cur_sel { '>' } else { ' ' });
				win.addstr(o);
				for _ in o.len() as i32 .. w {
					win.addch(' ');
				}
				win.addch(' ');
				win.addch('|');
			}

			win.mv(y + 1 + h+1, x);
			win.addch('\\');
			win.addch('-');
			win.hline('-', w);
			win.mv(y + 1 + h+1, x+w+2);
			win.addch('-');
			win.addch('/');
		}

		match win.getch()
		{
		Some(pc::Input::Character('q')) => return None,
		Some(pc::Input::KeyEnter) | Some(pc::Input::Character('\n')) =>
			if cur_sel == options.len() {
				return None;
			}
			else {
				return Some(cur_sel);
			},
		Some(pc::Input::KeyDown) | Some(pc::Input::Character('j')) =>
			if cur_sel <= options.len() {
				cur_sel += 1;
			}
			else {
				// Nope
			},
		Some(pc::Input::KeyUp) | Some(pc::Input::Character('k')) =>
			if cur_sel > 0 {
				cur_sel -= 1;
			}
			else {
				// Nope
			},
		_ => {},
		}
	}
}

fn draw_expression_nosel(win: &pc::Window, e: &Expression)
{
	use std::fmt::Write;
	write!(WindowFmt(win), "{}", e);
}
fn draw_expression(win: &pc::Window, line: &crate::Line)
{
	let (before, hilight, after,) = line.render_split();
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

