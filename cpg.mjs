import { CodeJar } from "codejar";
import PEG from "peggy";

const grammar = `
{
	const $ = options;

	const span = (c, t) => {
		const s = document.createElement('span');
		s.classList.add(c);
		if (typeof t === 'string') {
			s.appendChild(document.createTextNode(t));
		} else if (Array.isArray(t)) {
			t.forEach(ch => ch instanceof HTMLElement && s.appendChild(ch));
		} else if (t instanceof HTMLElement) {
			s.appendChild(t);
		}
		return s;
	};

	const STRENGTH = {
		req: 1001001000.0,
		strong: 1000000.0,
		med: 1000.0,
		weak: 1.0
	};

	const vars = new Map();
	const var_names = new Map();
	let default_strength = STRENGTH.med;

	$.reset();
}

program
	= e:program_
	{
		vars.forEach((elem, i) => elem.setAttribute('content', ' = ' + $.get_var(i)));
		@e.flat(Infinity)
	}
	;

program_
	= (WS / NL)* statement*
	;

statement
	= ((WS? (comment / (statement_ WS? comment?))) / WS) (NL / !.)
	;

statement_
	= dim
	/ use_strength
	/ constraint
	;

constraint
	= t:(expr WS? OP WS? expr) s:(WS? STRENGTH)?
	{
		const strength = s?.[1].n || default_strength;
		$[t[2].op](strength);
		@[t, s]
	}
	;

OP
	= "==" { const s = span('op', '=='); s.op = 'op_eq'; @s }
	/ ">=" { const s = span('op', '>='); s.op = 'op_gte'; @s }
	/ "<=" { const s = span('op', '<='); s.op = 'op_lte'; @s }
	;

expr
	= addsub_expr
	;

addsub_expr
	= muldiv_expr (addsub_expr_sub / addsub_expr_add)*
	;

addsub_expr_sub
	= t:(WS? SUB WS? muldiv_expr)
	{ $.op_sub(); @t }
	;

addsub_expr_add
	= t:(WS? ADD WS? muldiv_expr)
	{ $.op_add(); @t }
	;

SUB
	= "-"
	{ @span('op', '-') }
	;

ADD
	= "+"
	{ @span('op', '+') }
	;

muldiv_expr
	= term (muldiv_expr_mul / muldiv_expr_div)*
	;

muldiv_expr_mul
	= t:(WS? MUL WS? term)
	{ $.op_mul(); @t }
	;

muldiv_expr_div
	= t:(WS? DIV WS? term)
	{ $.op_div(); @t }
	;

MUL
	= "*"
	{ @span('op', '*') }
	;

DIV
	= "/"
	{ @span('op', '/') }
	;

term
	= clause
	/ const
	/ var_ref
	;

const
	= n:NUM
	{
		$.p_const(n.n);
		@n
	}
	;

var_ref
	= id:IDENT
	{
		const vid = var_names.get(id);
		if (vid === undefined) {
			error("not a variable: " + id);
		}
		$.p_var(vid);
		@span('id', id)
	}
	;

clause
	= OPAREN WS? expr WS? CPAREN
	;

OPAREN
	= "("
	{ @span('punc', '(') }
	;

CPAREN
	= ")"
	{ @span('punc', ')') }
	;

use_strength
	= USE WS use_strength_
	;

use_strength_
	= s:STRENGTH
	{
		default_strength = s.n;
		console.debug('DEFAULT STRENGTH', default_strength);
		@s
	}
	;

comment
	= c:$("#" [^\\n]*)
	{ @span('comment', c) }
	;

dim
	= VAR WS dim_
	;

dim_
	= name:IDENT spec:(WS NUM (WS STRENGTH)?)?
	{
		let id = $.dim();
		const display = span('varval', null);
		vars.set(id, display);
		var_names.set(name, id);
		if (spec) {
			spec = spec.flat(Infinity);
			const n = spec[1].n;
			const strength = spec[3]?.n || default_strength;
			$.suggest(id, n, strength);
		}
		@[span('id', name), spec, display]
	}
	;

STRENGTH
	= "!" n:(NUM_RAW / "req" / "med" / "strong" / "weak")
	{
		const [raw, real] = Array.isArray(n) ? n : [n, STRENGTH[n]];
		const s = span('strength', '!' + raw);
		s.n = real;
		@s
	}
	;

NUM
	= n:NUM_RAW
	{
		const [raw, v] = n;
		const s = span('const', raw);
		s.n = v;
		@s
	}
	;

NUM_RAW
	= n:$([0-9_']* ("." [0-9_']+) / [0-9_']+)
	{ @[n, parseFloat(n.replace(/[_']+/g, ''))] }
	;

IDENT
	= $([a-zA-Z_][a-zA-Z0-9_]*)
	;

VAR
	= "var"
	{ @span('kw', 'var') }
	;

USE
	= "use"
	{ @span('kw', 'use') }
	;

NL
	= c:$[\\n]+
	{ @span('ws', c) }
	;

WS
	= c:$[ \\t]+
	{ @span('ws', c) }
	;
`.replace(/@/g, "return ");

const parser = PEG.generate(grammar);

const root = document.getElementById("root");

const oldContents = sessionStorage.getItem("constraint-spec");
if (oldContents) {
	try {
		root.innerHTML = "";
		root.appendChild(document.createTextNode(JSON.parse(oldContents)));
	} catch (err) {
		console.warn("failed to load saved state:", err);
		console.warn("old state:");
		console.warn(oldContents);
	}
}

const jarOptions = {
	tab: "\t",
	indentOn: /$.^/, // never
	moveToNewLine: /$.^/, // never
	spellcheck: false,
	catchTab: true,
	preserveIndent: true,
	addClosing: false,
	history: true,
	window: window,
};

let API;

const highlight = (editor) => {
	let code = editor.textContent;
	let formatted;

	sessionStorage.setItem("constraint-spec", JSON.stringify(code));

	try {
		if (API) {
			let elems = parser.parse(code, API);
			if (elems && !Array.isArray(elems)) {
				elems = [elems];
			}
			formatted = document.createElement("span");
			elems.forEach((c) => c && formatted.appendChild(c));
		}
	} catch (err) {
		console.error("failed to parse:", err);
	}

	editor.innerHTML = formatted?.innerHTML ?? code;
};

new CodeJar(root, highlight, jarOptions);

const importObject = {
	env: { alert: (arg) => console.warn(arg) },
};

WebAssembly.instantiateStreaming(fetch("cpg.wasm"), importObject).then((obj) =>
	linkWasm(obj.instance)
);

function linkWasm(inst) {
	const proxied = {};

	for (const [k, v] of Object.entries(inst.exports)) {
		if (typeof v === "function" && !k.startsWith("_")) {
			((k, v) => {
				proxied[k] = function (...args) {
					let r = v.apply(this, args);
					console.debug("API:", k, args, r);
					return r;
				};
			})(k, v);
		}
	}

	API = proxied;

	highlight(root);
}
