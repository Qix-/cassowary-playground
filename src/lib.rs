#[macro_use]
extern crate lazy_static;
use casuarius::*;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Var(usize);
derive_syntax_for!(Var);

enum El {
	E(Expression<Var>),
	T(Term<Var>),
	V(Var),
	N(f64),
}

impl From<El> for Expression<Var> {
	fn from(el: El) -> Self {
		match el {
			El::E(v) => v,
			El::T(v) => v.into(),
			El::V(v) => v.into(),
			El::N(v) => v.into(),
		}
	}
}

impl From<Expression<Var>> for El {
	fn from(v: Expression<Var>) -> Self {
		Self::E(v)
	}
}

impl From<Term<Var>> for El {
	fn from(v: Term<Var>) -> Self {
		Self::T(v)
	}
}

impl From<Var> for El {
	fn from(v: Var) -> Self {
		Self::V(v)
	}
}

impl From<f64> for El {
	fn from(v: f64) -> Self {
		Self::N(v)
	}
}

#[derive(Default)]
struct State {
	solver: Solver<Var>,
	vars: Vec<Var>,
	stack: Vec<El>,
}

impl State {
	fn reset(&mut self) {
		self.solver.reset();
		self.vars.clear();
		self.stack.clear();
	}

	fn dim(&mut self) -> u32 {
		let id = self.vars.len();
		self.vars.push(Var(id));
		id as u32
	}

	fn suggest(&mut self, id: u32, v: f64, s: f64) -> i32 {
		let id = id as usize;

		if id >= self.vars.len() {
			return -1;
		}

		// Work around dumb rule that never made any sense
		// in the original Kiwi implementation that got
		// copied over to this implementation.
		let s = if s == REQUIRED { s - 0.0001f64 } else { s };

		if self.solver.add_edit_variable(self.vars[id], s).is_err() {
			return -2;
		}

		if self.solver.suggest_value(self.vars[id], v).is_err() {
			return -3;
		}

		return 0;
	}

	fn get_var(&self, id: u32) -> f64 {
		let id = id as usize;
		if id >= self.vars.len() {
			return 0.0;
		}
		self.solver.get_value(self.vars[id])
	}

	fn p_var(&mut self, id: u32) -> i32 {
		let id = id as usize;
		if id >= self.vars.len() {
			return -1;
		}
		self.stack.push(El::V(self.vars[id]));
		return 0;
	}

	fn p_const(&mut self, v: f64) -> i32 {
		self.stack.push(El::N(v));
		return 0;
	}

	fn op_eq(&mut self, s: f64) -> i32 {
		if self.stack.len() != 2 {
			self.stack.clear();
			return -1;
		}

		let r: Expression<Var> = self.stack.pop().unwrap().into();
		let l: Expression<Var> = self.stack.pop().unwrap().into();
		self.stack.clear();

		if self
			.solver
			.add_constraint(l.is(r).with_strength(s))
			.is_err()
		{
			return -2;
		}

		return 0;
	}

	fn op_lte(&mut self, s: f64) -> i32 {
		if self.stack.len() != 2 {
			self.stack.clear();
			return -1;
		}

		let r: Expression<Var> = self.stack.pop().unwrap().into();
		let l: Expression<Var> = self.stack.pop().unwrap().into();
		self.stack.clear();

		if self
			.solver
			.add_constraint(l.is_le(r).with_strength(s))
			.is_err()
		{
			return -2;
		}

		return 0;
	}

	fn op_gte(&mut self, s: f64) -> i32 {
		if self.stack.len() != 2 {
			self.stack.clear();
			return -1;
		}

		let r: Expression<Var> = self.stack.pop().unwrap().into();
		let l: Expression<Var> = self.stack.pop().unwrap().into();
		self.stack.clear();

		if self
			.solver
			.add_constraint(l.is_ge(r).with_strength(s))
			.is_err()
		{
			return -2;
		}

		return 0;
	}

	fn op_mul(&mut self) -> i32 {
		if self.stack.len() < 2 {
			return -1;
		}

		let r = self.stack.pop().unwrap();
		let l = self.stack.pop().unwrap();

		let v: El = match l {
			El::E(l) => match r {
				El::N(r) => (l * r).into(),
				_ => return -2,
			},
			El::T(l) => match r {
				El::N(r) => (l * r).into(),
				_ => return -2,
			},
			El::V(l) => match r {
				El::N(r) => (l * r).into(),
				_ => return -2,
			},
			El::N(l) => match r {
				El::E(r) => (l * r).into(),
				El::T(r) => (l * r).into(),
				El::V(r) => (l * r).into(),
				El::N(r) => (l * r).into(),
			},
		};

		self.stack.push(v.into());

		return 0;
	}

	fn op_div(&mut self) -> i32 {
		if self.stack.len() < 2 {
			return -1;
		}

		let r = self.stack.pop().unwrap();
		let l = self.stack.pop().unwrap();

		let v: El = match l {
			El::E(l) => match r {
				El::N(r) => (l / r).into(),
				_ => return -2,
			},
			El::T(l) => match r {
				El::N(r) => (l / r).into(),
				_ => return -2,
			},
			El::V(l) => match r {
				El::N(r) => (l / r).into(),
				_ => return -2,
			},
			El::N(l) => match r {
				El::N(r) => (l / r).into(),
				_ => return -2,
			},
		};

		self.stack.push(v.into());

		return 0;
	}

	fn op_add(&mut self) -> i32 {
		if self.stack.len() < 2 {
			return -1;
		}

		let r = self.stack.pop().unwrap();
		let l = self.stack.pop().unwrap();

		let v: El = match l {
			El::E(l) => match r {
				El::E(r) => (l + r).into(),
				El::T(r) => (l + r).into(),
				El::V(r) => (l + r).into(),
				El::N(r) => (l + r).into(),
			},
			El::T(l) => match r {
				El::E(r) => (l + r).into(),
				El::T(r) => (l + r).into(),
				El::V(r) => (l + r).into(),
				El::N(r) => (l + r).into(),
			},
			El::V(l) => match r {
				El::E(r) => (l + r).into(),
				El::T(r) => (l + r).into(),
				El::V(r) => (l + r).into(),
				El::N(r) => (l + r).into(),
			},
			El::N(l) => match r {
				El::E(r) => (l + r).into(),
				El::T(r) => (l + r).into(),
				El::V(r) => (l + r).into(),
				El::N(r) => (l + r).into(),
			},
		};

		self.stack.push(v.into());

		return 0;
	}

	fn op_sub(&mut self) -> i32 {
		if self.stack.len() < 2 {
			return -1;
		}

		let r = self.stack.pop().unwrap();
		let l = self.stack.pop().unwrap();

		let v: El = match l {
			El::E(l) => match r {
				El::E(r) => (l - r).into(),
				El::T(r) => (l - r).into(),
				El::V(r) => (l - r).into(),
				El::N(r) => (l - r).into(),
			},
			El::T(l) => match r {
				El::E(r) => (l - r).into(),
				El::T(r) => (l - r).into(),
				El::V(r) => (l - r).into(),
				El::N(r) => (l - r).into(),
			},
			El::V(l) => match r {
				El::E(r) => (l - r).into(),
				El::T(r) => (l - r).into(),
				El::V(r) => (l - r).into(),
				El::N(r) => (l - r).into(),
			},
			El::N(l) => match r {
				El::E(r) => (l - r).into(),
				El::T(r) => (l - r).into(),
				El::V(r) => (l - r).into(),
				El::N(r) => (l - r).into(),
			},
		};

		self.stack.push(v.into());

		return 0;
	}
}

lazy_static! {
	static ref STATE: Mutex<State> = Mutex::new(State::default());
}

#[no_mangle]
pub extern "C" fn reset() -> i32 {
	STATE.lock().unwrap().reset();
	0
}

#[no_mangle]
pub extern "C" fn dim() -> u32 {
	STATE.lock().unwrap().dim()
}

#[no_mangle]
pub extern "C" fn suggest(id: u32, v: f64, s: f64) -> i32 {
	STATE.lock().unwrap().suggest(id, v, s)
}

#[no_mangle]
pub extern "C" fn get_var(id: u32) -> f64 {
	STATE.lock().unwrap().get_var(id)
}

#[no_mangle]
pub extern "C" fn p_var(id: u32) -> i32 {
	STATE.lock().unwrap().p_var(id)
}

#[no_mangle]
pub extern "C" fn p_const(v: f64) -> i32 {
	STATE.lock().unwrap().p_const(v)
}

#[no_mangle]
pub extern "C" fn op_eq(s: f64) -> i32 {
	STATE.lock().unwrap().op_eq(s)
}

#[no_mangle]
pub extern "C" fn op_gte(s: f64) -> i32 {
	STATE.lock().unwrap().op_gte(s)
}

#[no_mangle]
pub extern "C" fn op_lte(s: f64) -> i32 {
	STATE.lock().unwrap().op_lte(s)
}

#[no_mangle]
pub extern "C" fn op_mul() -> i32 {
	STATE.lock().unwrap().op_mul()
}

#[no_mangle]
pub extern "C" fn op_div() -> i32 {
	STATE.lock().unwrap().op_div()
}

#[no_mangle]
pub extern "C" fn op_add() -> i32 {
	STATE.lock().unwrap().op_add()
}

#[no_mangle]
pub extern "C" fn op_sub() -> i32 {
	STATE.lock().unwrap().op_sub()
}
