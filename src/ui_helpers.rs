use crate::expression::Expression;

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
		Expression::SubNode(sn) => get_level_size_node(sn,  path,last_idx,  path_pos),
		Expression::Literal(_v) => { assert!(path_pos == path.len()); None },	// TODO: Impossible?
		Expression::Variable(_v) => { assert!(path_pos == path.len()); None },
		}
	}
	fn get_level_size_node(e: &crate::expression::ExprNode, path: &[usize], last_idx: usize, path_pos: usize) -> Option<usize>
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

//pub fn render_expression(e: &crate::expression::Expression, sel: &Selection, seg_mask: u8) -> (String, String, String)
//{
//}

pub fn extract_subexpression(e: &Expression, sel: &Selection) -> Expression
{
	fn h_expr(e: &crate::expression::Expression, sel: &Selection, path_pos: usize) -> Expression
	{
		match e
		{
		Expression::SubNode(sn) => h_node(sn, sel, path_pos),
		Expression::Literal(_v) => e.clone(),
		Expression::Variable(_v) => e.clone(),
		}
	}
	fn h_node(e: &crate::expression::ExprNode, sel: &Selection, path_pos: usize) -> Expression
	{
		assert!(path_pos <= sel.path.len());
		if path_pos < sel.path.len() {
			let idx = sel.path[path_pos];
			assert!( idx < e.values.len() );
			h_expr( &e.values[idx].val, sel, path_pos+1 )
		}
		else if sel.first == sel.last {
			// TODO: Negations?
			e.values[sel.first].val.clone()
		}
		else {
			let mut rv = crate::expression::ExprNode {
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

pub fn split_expression(e: &crate::expression::Expression, sel: &Selection) -> (String, String, String)
{
	let mut sink = RenderSink::new();
	draw_sub_expression(&mut sink, e, sel, 0);
	assert!(sink.cur_buf != 1);
	(
		::std::mem::replace(&mut sink.buffers[0], String::new()),
		::std::mem::replace(&mut sink.buffers[1], String::new()),
		::std::mem::replace(&mut sink.buffers[2], String::new()),
		)
}

fn draw_sub_expression(sink: &mut RenderSink, e: &crate::expression::Expression, sel: &Selection, path_pos: usize)
{
	match e
	{
	Expression::SubNode(sn) => draw_sub_expression_node(sink, sn, sel, path_pos),
	Expression::Literal(v) => sink.put(&v),
	Expression::Variable(v) => sink.put(&v),
	}
}
fn draw_sub_expression_node(sink: &mut RenderSink, e: &crate::expression::ExprNode, sel: &Selection, path_pos: usize)
{
	//println!("path_pos={} e={:?}", path_pos, e);
	use crate::expression::Op;

	for (i,v) in Iterator::enumerate(e.values.iter())
	{

		if i == 0
		{
		}
		else
		{
			match e.operation
			{
			Op::Add => sink.put(if v.negated { "" } else { "+" }),
			Op::Multiply => sink.put("*"),
			Op::Divide   => sink.put("/"),
			Op::Exponent => sink.put("^"),
			Op::Equality => sink.put("="),
			}
		}

		if path_pos == sel.path.len() && i == sel.first {
			sink.start_hilight();
		}

		if v.negated {
			sink.put("-");
		}

		let needs_parens = v.val.needs_parens(e.operation);
		if needs_parens {
			sink.put("(");
		}
		draw_sub_expression(sink, &v.val, sel, if path_pos < sel.path.len() && sel.path[path_pos] == i { path_pos + 1 } else { !0 });
		if needs_parens {
			sink.put(")");
		}

		if path_pos == sel.path.len() && i == sel.last {
			sink.end_hilight();
		}
	}

	assert!(!(path_pos == sel.path.len() && sink.hilight_active()), "Path was invalid, didn't terminate hilight");
}
