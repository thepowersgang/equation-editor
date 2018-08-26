use crate::expression::Expression;
use crate::expression::ExprNode;

#[derive(Clone)]
pub struct Selection {
	pub path: Vec<usize>,
	pub first: usize,
	pub last: usize,
}
impl std::fmt::Debug for Selection {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?} {}-{}", self.path, self.first, self.last)
	}
}

impl Selection
{
	fn get_cur_size(&self, e: &Expression) -> usize {
		if let Some( (&last, s) ) = self.path.split_last()
		{
			get_level_size(e, s, last).unwrap()
		}
		else
		{
			get_level_size(e, &[], !0).unwrap()
		}
	}
	pub fn move_out(&mut self, _e: &Expression) -> bool
	{
		if let Some(v) = self.path.pop()
		{
			self.first = v;
			self.last = v;
			true
		}
		else
		{
			false
		}
	}
	pub fn move_in(&mut self, e: &Expression) -> bool
	{
		if self.first < self.last
		{
			self.last = self.first;
			true
		}
		else if let Some(size) = get_level_size(e, &self.path, self.first)
		{
			assert!(size > 0);
			self.path.push(self.first);
			self.first = 0;
			self.last = 0;
			true
		}
		else
		{
			false
		}
	}

	pub fn shift_right(&mut self, e: &Expression) -> bool
	{
		// NOTE: This forces the right to equal the left
		let v = self.get_cur_size(e);
		if self.first < v-1 {
			self.first += 1;
			self.last = self.first;
			true
		}
		else {
			self.last = self.first;
			false
		}
	}

	pub fn shift_left(&mut self, _e: &Expression) -> bool
	{
		// NOTE: This forces the right to equal the left
		if self.first > 0 {
			self.first -= 1;
			self.last = self.first;
			true
		}
		else {
			self.last = self.first;
			false
		}
	}

	pub fn expand_right(&mut self, e: &Expression) -> bool
	{
		let v = self.get_cur_size(e);
		if self.last < v-1 {
			self.last += 1;
			true
		}
		else {
			false
		}
	}
	pub fn expand_left(&mut self, _e: &Expression) -> bool
	{
		if self.first > 0 {
			self.first -= 1;
			true
		}
		else {
			false
		}
	}

	pub fn shrink_right(&mut self, _e: &Expression) -> bool
	{
		if self.last > self.first {
			self.last -= 1;
			true
		}
		else {
			false
		}
	}
	pub fn shrink_left(&mut self, _e: &Expression) -> bool
	{
		if self.first < self.last {
			self.first += 1;
			true
		}
		else {
			false
		}
	}
}

pub struct RenderSink
{
	cur_buf: usize,
	buffers: [String; 3],
}
impl RenderSink
{
	pub fn new() -> RenderSink {
		RenderSink {
			cur_buf: 0,
			buffers: [String::new(), String::new(), String::new()],
			}
	}
	pub fn put(&mut self, v: impl std::fmt::Display)
	{
		use std::fmt::Write;
		write!(&mut self.buffers[self.cur_buf], "{}", v);
	}
	pub fn hilight_active(&self) -> bool {
		self.cur_buf == 1
	}
	pub fn start_hilight(&mut self) {
		assert!(self.cur_buf == 0);
		self.cur_buf = 1;
	}
	pub fn end_hilight(&mut self) {
		assert!(self.cur_buf == 1);
		self.cur_buf = 2;
	}
}

fn get_level_size(e: &Expression, path: &[usize], last_idx: usize) -> Option<usize>
{
	fn get_level_size_expr(e: &Expression, path: &[usize], last_idx: usize,  path_pos: usize) -> Option<usize>
	{
		match e
		{
		Expression::Negative(sn) =>
			if path_pos == path.len() {
				assert!(last_idx == 0);
				Some(1)
			}
			else {
				get_level_size_expr(sn, path, last_idx, path_pos+1)
			},
		Expression::SubNode(sn) => get_level_size_node(sn,  path,last_idx,  path_pos),
		Expression::Literal(_v) => { assert!(path_pos == path.len()); None },	// TODO: Impossible?
		Expression::Variable(_v) => { assert!(path_pos == path.len()); None },
		}
	}
	fn get_level_size_node(e: &ExprNode, path: &[usize], last_idx: usize, path_pos: usize) -> Option<usize>
	{
		assert!(path_pos <= path.len());
		if path_pos == path.len() {
			if last_idx == !0 {
				assert!(path_pos == 0);
				return Some( e.values.len() );
			}
			else {
				let idx = last_idx;
				assert!( idx < e.values.len() );
				match e.values[idx].val
				{
				Expression::Negative(_) => Some(1),
				Expression::SubNode(ref sn) => Some(sn.values.len()),
				_ => None,
				}
			}
		}
		else {
			let idx = path[path_pos];
			assert!( idx < e.values.len() );
			get_level_size_expr( &e.values[idx].val, path, last_idx, path_pos+1 )
		}
	}
	get_level_size_expr(e, path, last_idx,  0)
}

pub fn extract_subexpression(e: &Expression, sel: &Selection) -> Expression
{
	fn h_expr(e: &Expression, sel: &Selection, path_pos: usize) -> Expression
	{
		match e
		{
		Expression::Negative(e) =>
			if path_pos == sel.path.len() {
				(**e).clone()
			}
			else {
				h_expr(e, sel, path_pos+1)
			},
		Expression::SubNode(sn) => h_node(sn, sel, path_pos),
		Expression::Literal(_v) => e.clone(),
		Expression::Variable(_v) => e.clone(),
		}
	}
	fn h_node(e: &ExprNode, sel: &Selection, path_pos: usize) -> Expression
	{
		assert!(path_pos <= sel.path.len());
		if path_pos < sel.path.len() {
			let idx = sel.path[path_pos];
			assert!( idx < e.values.len() );
			h_expr( &e.values[idx].val, sel, path_pos+1 )
		}
		// Single expression
		else if sel.first == sel.last {
			e.values[sel.first].val.clone()
		}
		// Range of expressions
		// TODO: When copying, should the operator be included?
		else {
			let mut rv = ExprNode {
				operation: e.operation,
				values: vec![],
				};
			for se in e.values[sel.first .. sel.last+1].iter() {
				rv.values.push( se.clone() );
			}
			Expression::SubNode( rv )
		}
	}
	h_expr(e, sel, 0)
}

pub fn replace_subexpression(e: &mut Expression, sel: &mut Selection, new_e: Expression)
{
	fn h_expr(e: &mut Expression, sel: &mut Selection, path_pos: usize, new_e: Expression, simplify: bool)
	{
		match e
		{
		Expression::Negative(e) =>
			if path_pos == sel.path.len() {
				**e = new_e;
			}
			else {
				h_expr(e, sel, path_pos+1, new_e, simplify)
			},
		Expression::SubNode(sn) => h_node(sn, sel, path_pos, new_e, simplify),
		Expression::Literal(_v) => panic!(""),
		Expression::Variable(_v) => panic!(""),
		}
	}
	fn h_node(e: &mut ExprNode, sel: &mut Selection, path_pos: usize, mut new_e: Expression, simplify: bool)
	{
		assert!(path_pos <= sel.path.len());
		if path_pos < sel.path.len() {
			let idx = sel.path[path_pos];
			assert!( idx < e.values.len() );
			h_expr( &mut e.values[idx].val, sel, path_pos+1, new_e, simplify )
		}
		// Single expression
		else if sel.first == sel.last {
			match new_e
			{
			Expression::SubNode(ref isn) if simplify && isn.operation == e.operation => {
				panic!("TODO: Merge");
				},
			_ => {
				e.values[sel.first].val = new_e;
				}
			}
		}
		// Range of expressions
		else {
			let is_dst_inv = e.values[sel.first].inverse;

			match new_e
			{
			Expression::SubNode(ref mut sn) if sn.operation == e.operation => {
				// has sub-values?
				let len = sn.values.len();
				e.values.splice(sel.first .. sel.last+1,  sn.values.drain(..));
				sel.last = sel.first + len - 1;
				},
			_ => {
				e.values.drain(sel.first .. sel.last+1);
				e.values.insert(sel.first, crate::expression::SubExpression { inverse: is_dst_inv, val: new_e });
				sel.last = sel.first;
				}
			}
			// TODO: Should this update the selection too?
		}
	}
	h_expr(e, sel, 0, new_e, /*simplify=*/false)
}

pub fn split_expression(e: &Expression, sel: &Selection) -> (String, String, String)
{
	let mut sink = RenderSink::new();
	
	fn h_expr(sink: &mut RenderSink, e: &Expression, sel: &Selection, path_pos: usize)
	{
		match e
		{
		Expression::Negative(e) => {
			sink.put("-");
			let needs_parens = match **e
				{
				Expression::Literal(_) | Expression::Variable(_) => false,
				_ => true,
				};
			if path_pos < sel.path.len() {
				assert!(sel.path[path_pos] == 0);
			}
			if path_pos == sel.path.len() {
				assert!(sel.first == 0);
				assert!(sel.last == 0);
			}
			if path_pos == sel.path.len() {
				sink.start_hilight();
			}
			if needs_parens {
				sink.put("(");
			}
			h_expr(sink, e, sel, if path_pos < sel.path.len() { path_pos+1 } else { !0 });
			if needs_parens {
				sink.put(")");
			}
			if path_pos == sel.path.len() {
				sink.end_hilight();
			}
			},
		Expression::SubNode(sn) => h_node(sink, sn, sel, path_pos),
		Expression::Literal(v) => sink.put(&v),
		Expression::Variable(v) => sink.put(&v),
		}
	}
	fn h_node(sink: &mut RenderSink, e: &ExprNode, sel: &Selection, path_pos: usize)
	{
		for (i,v) in Iterator::enumerate(e.values.iter())
		{
			if i == 0
			{
			}
			else
			{
				match e.operation
				{
				crate::expression::Op::AddSub => sink.put(if v.inverse { "-" } else { "+" }),
				crate::expression::Op::MulDiv => sink.put(if v.inverse { "/" } else { "*" }),
				crate::expression::Op::ExpRoot => sink.put("^"),
				crate::expression::Op::Equality => sink.put("="),
				}
			}

			if path_pos == sel.path.len() && i == sel.first {
				sink.start_hilight();
			}

			let needs_parens = v.val.needs_parens(e.operation);
			if needs_parens {
				sink.put("(");
			}
			h_expr(sink, &v.val, sel, if path_pos < sel.path.len() && sel.path[path_pos] == i { path_pos + 1 } else { !0 });
			if needs_parens {
				sink.put(")");
			}

			if path_pos == sel.path.len() && i == sel.last {
				sink.end_hilight();
			}
		}

		assert!(!(path_pos == sel.path.len() && sink.hilight_active()), "Path was invalid, didn't terminate hilight");
	}

	h_expr(&mut sink, e, sel, 0);
	assert!(sink.cur_buf != 1);
	(
		::std::mem::replace(&mut sink.buffers[0], String::new()),
		::std::mem::replace(&mut sink.buffers[1], String::new()),
		::std::mem::replace(&mut sink.buffers[2], String::new()),
		)
}
