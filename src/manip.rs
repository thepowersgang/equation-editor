//!
//!
//!

use crate::expression::{Expression, SubExpression, Op, ExprNode};

// Convert an expression into a common form
pub fn normalise(e: Expression) -> Expression
{
	// - Sort
	// - Multiply all into `ax + bx` form
	// - Sort all components
	// - Collect exponents
	e
}

pub fn factorise_trailing(e: Expression) -> Option<Expression>
{
	factorise_int(e, Factorise::Trailing)
}
pub fn factorise_leading(e: Expression) -> Option<Expression>
{
	factorise_int(e, Factorise::Leading)
}
pub fn factorise_all(e: Expression) -> Option<Expression>
{
	factorise_int(e, Factorise::All)
}
enum Factorise
{
	All,
	Leading,
	Trailing,
}
fn factorise_int(e: Expression, ty: Factorise) -> Option<Expression>
{
	if let Expression::SubNode(mut sn) = e
	{
		// Needs to be an addition
		if sn.operation == Op::AddSub
		{
			#[derive(Debug,Copy,Clone)]
			enum Ent<'a> {
				Direct(&'a Expression),
				DivMul(&'a SubExpression),
			}
			impl<'a> PartialEq for Ent<'a> {
				fn eq(&self, e: &Self) -> bool {
					match self
					{
					Ent::Direct(v) =>
						match e
						{
						Ent::Direct(v2) => *v == *v2,
						Ent::DivMul(v2) => !v2.inverse && **v == v2.val,
						},
					Ent::DivMul(v) =>
						match e
						{
						Ent::Direct(v2) => !v.inverse && v.val == **v2,
						Ent::DivMul(v2) => *v == *v2,
						},
					}
				}
			}
			match ty
			{
			Factorise::All => {
				let common: Vec<SubExpression> = {
					// Enumerate all components and collect into comparable types
					let mut items = vec![ ];
					for v in sn.values.iter()
					{
						let this_items: Vec<Ent> = match v.val
							{
							Expression::SubNode(ref isn) if isn.operation == Op::MulDiv => {
								// Iterate all entries, add to list for this item
								isn.values.iter().map( |i| Ent::DivMul(i) ).collect()
								},
							_ => {
								// Add to list for this item
								vec![ Ent::Direct(&v.val) ]
								}
							};
						// TODO: Assert that there's no duplicates.
						items.push( this_items );
					}

					// Find items common in all
					let mut common_ents = vec![];
					for v in items[0].iter()
					{
						let mut is_missing = false;
						for l in items.iter()
						{
							if l.iter().find(|x| *x == v).is_none() {
								is_missing = true;
								break;
							}
						}
						if !is_missing {
							common_ents.push(match *v
								{
								Ent::Direct(e) => SubExpression { inverse: false, val: e.clone() },
								Ent::DivMul(se) => se.clone(),
								});
						}
					}
					common_ents
					};

				// Replace these common items
				if common.len() > 0
				{
					for ent in sn.values.iter_mut()
					{
						let v = match ent.val
							{
							Expression::SubNode(ref mut sn) =>
								if sn.values.len() == common.len() {
									Some( Expression::Literal(1.) )
								}
								else {
									// Retain all values that aren't in the common list
									// TODO: What about duplicates? Only the first should be removed. (or, just assert that above?)
									sn.values.retain(|v| common.iter().any(|x| x != v));

									if sn.values[0].inverse {
										sn.values.insert(0, SubExpression { inverse: false, val: Expression::Literal(1.) });
										None
									}
									else if sn.values.len() == 1 {
										Some( sn.values.pop().unwrap().val )
									}
									else {
										None
									}
								},
							_ => {
								assert!( common.len() == 1 );
								assert!( common[0].inverse == false );
								assert!( common[0].val == ent.val );
								Some( Expression::Literal(1.) )
								},
							};

						if let Some(v) = v {
							ent.val = v;
						}
					}

					let mut common = common;
					if common[0].inverse {
						common.insert(0, SubExpression { inverse: false, val: Expression::Literal(1.) });
					}
					common.push( SubExpression { inverse: false, val: Expression::SubNode(sn) } );

					Some(Expression::SubNode(ExprNode {
						operation: Op::MulDiv,
						values: common,
						}))
				}
				else
				{
					None
				}
				},
			Factorise::Leading => {
				fn get_first(v: &SubExpression)->&Expression {
					match v.val
					{
					Expression::SubNode(ref isn) if isn.operation == Op::MulDiv => {
						let i = isn.values.first().unwrap();
						assert!( !i.inverse );
						&i.val
						},
					ref e @ _ => e,
					}
				}
				let item = get_first(sn.values.first().unwrap()).clone();
				if sn.values.iter().skip(1).all(|v| *get_first(v) == item) {
					// All equal!
					// - Remove the leading from all entries (possibly replacing with 1)
					for ent in sn.values.iter_mut()
					{
						let v = match ent.val
							{
							Expression::SubNode(ref mut sn) if sn.values[1].inverse => {
								// Lead to `1/foo`
								sn.values[0].val = Expression::Literal(1.);
								None
								},
							Expression::SubNode(ref mut sn) if sn.values.len() > 2 => {
								assert!( !sn.values[1].inverse );
								sn.values.remove(0);
								None
								},
							Expression::SubNode(ref mut sn) => {
								assert!( sn.values.len() == 2 );
								assert!( !sn.values[1].inverse );
								Some( sn.values.pop().unwrap().val )
								},
							_ => { Some( Expression::Literal(1.) ) },
							};
						if let Some(v) = v {
							ent.val = v;
						}
					}
					// Wrap (caller should run a simplify)
					Some( Expression::SubNode(ExprNode {
						operation: Op::MulDiv,
						values: vec![
							SubExpression { inverse: false, val: item.clone() },
							SubExpression { inverse: false, val: Expression::SubNode(sn) },
							],
						}) )
				}
				else {
					None
				}
				},
			Factorise::Trailing => {
				fn get_last(v: &SubExpression)->Ent {
					match v.val
					{
					Expression::SubNode(ref isn) if isn.operation == Op::MulDiv => {
						let i = isn.values.last().unwrap();
						Ent::DivMul(i)
						},
					ref e @ _ => Ent::Direct(e),
					}
				}
				if let Some(new_sub_item) = {
					let item = get_last( sn.values.first().unwrap() );
					if sn.values.iter().skip(1).all(|v| get_last(v) == item) {
						Some(match item
							{
							Ent::Direct(v) => SubExpression { inverse: false, val: v.clone() },
							Ent::DivMul(v) => v.clone(),
							})
					}
					else {
						None
					}
					}
				{
					// All equal!
					// - Remove the trailing from all entries (possibly replacing with 1)
					for ent in sn.values.iter_mut()
					{
						let v = match ent.val
							{
							Expression::SubNode(ref mut sn) if sn.values.len() > 2 => {
								sn.values.pop();
								None
								},
							Expression::SubNode(ref mut sn) => {
								assert!( sn.values.len() == 2 );
								sn.values.pop();
								Some( sn.values.pop().unwrap().val )
								},
							_ => { Some( Expression::Literal(1.) ) },
							};
						if let Some(v) = v {
							ent.val = v;
						}
					}
					// Wrap (caller should run a simplify)
					Some( Expression::SubNode(ExprNode {
						operation: Op::MulDiv,
						values: vec![
							SubExpression { inverse: false, val: Expression::SubNode(sn) },
							new_sub_item,
							],
						}) )
				}
				else {
					None
				}
				},
			}
		}
		else
		{
			None
		}
	}
	else
	{
		None
	}
}

