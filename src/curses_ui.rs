//!
//! pancurses backed UI
//!

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
		LineSelectExt,
		ExprPick,	// Pick a single sub-expression
		ExprSelect,	// Select a range of sub-expressions
		ExprMove,	// Cursor keys move the insertion point around the same level as the selection, esc ends
	}
	enum AltMode
	{
		None,
		CutPaste(usize),	// Selecting a location to paste a line
		CopyPaste(usize),	// Selecting a location to paste a copy of a line
	}
	#[derive(PartialEq,Debug)]
	enum Redraw
	{
		None,
		Current,
		All,
	}
	let mut cur_line = 0;
	let mut mode = InputMode::LineSelect;
	let mut alt_mode = AltMode::None;

	let mut last_line = 0;
	let mut redraw = Redraw::All;
	loop {
		if cur_line != last_line && redraw != Redraw::All {
			window.mv(last_line as i32, 2);
			draw_expression_nosel(&window, &lines[last_line].expr);
			last_line = cur_line;
			redraw = Redraw::Current;
		}
		if redraw == Redraw::All
		{
			window.clear();
			for (i, line) in lines.iter().enumerate()
			{
				window.mv(i as i32, 2);
				draw_expression_nosel(&window, &line.expr);
			}
		}
		if redraw != Redraw::None
		{
			let line = &lines[cur_line];
			window.mv(cur_line as i32, 2);
			window.clrtoeol();
			if mode == InputMode::LineSelect || mode == InputMode::LineSelectExt { 
				window.attron(pc::Attribute::Bold);
				draw_expression_nosel(&window, &line.expr);
				window.attroff(pc::Attribute::Bold);
			}
			else {
				draw_expression(&window, &line.expr, &line.sel);
			}
			
			{
				let v = match mode
					{
					InputMode::LineSelect => "LINE",
					InputMode::LineSelectExt => "LINE (ALT)",
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
			if let Some(opid) = show_menu_modal(&window, &["Extract common factors", "Distribute", "Edit expression"])
			{
				log!(window, "option {:?}", opid);
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

		// TODO: Should this use a clipboard instead?
		Some(pc::Input::Character('D')) | Some(pc::Input::Character('d')) =>
			match mode
			{
			InputMode::LineSelect => {
				mode = InputMode::LineSelectExt;
				alt_mode = AltMode::CutPaste(cur_line);
				redraw = Redraw::Current;
				},
			InputMode::ExprSelect => {
				// TODO: Cut/Paste expressions
				},
			_ => {},
			},
		Some(pc::Input::Character('Y')) | Some(pc::Input::Character('y')) =>
			match mode
			{
			InputMode::LineSelect => {
				mode = InputMode::LineSelectExt;
				alt_mode = AltMode::CopyPaste(cur_line);
				redraw = Redraw::Current;
				},
			InputMode::ExprSelect => {
				// TODO: Copy/paste expressions
				},
			_ => {},
			},
		Some(pc::Input::Character('P')) =>
			match mode
			{
			InputMode::LineSelectExt => {
				match alt_mode
				{
				AltMode::None => panic!(""),
				AltMode::CopyPaste(src) => {
					let v = lines[src].clone();
					lines.insert(cur_line, v);
					mode = InputMode::LineSelect;
					redraw = Redraw::All;
					},
				AltMode::CutPaste(src) => {
					let v = lines.remove(src);
					if src < cur_line {
						cur_line -= 1;
					}
					lines.insert(cur_line, v);
					mode = InputMode::LineSelect;
					redraw = Redraw::All;
					},
				}
				alt_mode = AltMode::None;
				},
			_ => {},
			},
		Some(pc::Input::Character('p')) =>
			match mode
			{
			InputMode::LineSelectExt => {
				match alt_mode
				{
				AltMode::None => panic!(""),
				AltMode::CopyPaste(src) => {
					let v = lines[src].clone();
					lines.insert(cur_line+1, v);
					cur_line += 1;
					mode = InputMode::LineSelect;
					redraw = Redraw::All;
					},
				AltMode::CutPaste(src) => {
					if src != cur_line
					{
						let v = lines.remove(src);
						if src <= cur_line {
							cur_line -= 1;
						}
						lines.insert(cur_line+1, v);
						cur_line += 1;
					}
					mode = InputMode::LineSelect;
					redraw = Redraw::All;
					},
				}
				alt_mode = AltMode::None;
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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
			InputMode::LineSelect | InputMode::LineSelectExt => {
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

