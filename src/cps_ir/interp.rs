use super::{Atom, Cont, IR, Value, builtin_call};
use std::collections::HashMap;

pub struct Store<'a> {
    pub mem: Vec<Value<'a>>,
    pub count: usize,
}

impl<'a> Store<'a> {
    pub fn new() -> Store<'a> {
        Store {
            mem: vec![],
            count: 0,
        }
    }
    pub fn alloc(&mut self, val: Value<'a>) -> usize {
        let site = self.count;
        self.mem.push(val);
        self.count += 1;
        site
    }
    pub fn get(&self, addr: usize) -> Value<'a> {
        self.mem
            .get(addr)
            .expect("store: address out of bound")
            .clone()
    }
    pub fn get_count(&self) -> usize {
        self.count
    }
    pub fn set_mem(&mut self, idx: usize, v: Value<'a>) -> () {
        self.mem[idx] = v;
    }
}

pub fn interp_atom<'a>(
    atom: &'a Atom,
    env: &HashMap<&'a str, usize>,
    store: &Store<'a>,
) -> Value<'a> {
    match atom {
        Atom::Var(name) => match env.get(name.as_str()) {
            Some(addr) => store.get(*addr),
            None => panic!("unbound variable: {name}"),
        },
        Atom::I32(v) => Value::I32(v.clone()),
        Atom::I64(v) => Value::I64(v.clone()),
        Atom::U32(v) => Value::U32(v.clone()),
        Atom::U64(v) => Value::U64(v.clone()),
        Atom::Bool(v) => Value::Bool(v.clone()),
        Atom::Char(v) => Value::Char(v.clone()),
        Atom::StringLiteral(v) => Value::StringLiteral(v.clone()),
        Atom::Lam(_label, args, ir) => Value::Clo(args, ir, env.clone()),
    }
}

pub fn apply_cont_by_name<'a>(
    cont_name: &'a str,
    values: Vec<Value<'a>>,
    env: HashMap<&'a str, usize>,
    store: &mut Store<'a>,
) -> Value<'a> {
    let Some(cont_addr) = env.get(cont_name) else {
        panic!("unbound continuation: {cont_name}");
    };
    match store.get(*cont_addr) {
        Value::Cont(args, body, env) => {
            let mut new_env = env;
            let mut values_ = values;
            for (i, val) in values_.drain(..).enumerate() {
                new_env.insert(&args[i], store.alloc(val));
            }
            interp(body, new_env, store)
        }
        _ => {
            panic!("apply_cont: not a continuation");
        }
    }
}

pub fn apply_cont<'a>(
    cont: &'a Cont,
    values: Vec<Value<'a>>,
    env: HashMap<&'a str, usize>,
    store: &mut Store<'a>,
) -> Value<'a> {
    match cont {
        Cont::Named(cont_name) => apply_cont_by_name(&cont_name, values, env, store),
        Cont::Return => values[0].clone(),
    }
}

pub fn interp<'a>(ir: &'a IR, env: HashMap<&'a str, usize>, store: &mut Store<'a>) -> Value<'a> {
    match ir {
        IR::LetCont(_label, cont_name, args, cont_body, body) => {
            let mut new_env = env.clone();
            new_env.insert(cont_name, store.alloc(Value::Cont(args, cont_body, env)));
            interp(body, new_env, store)
        }
        IR::Let(_label, bind, prim, args, body) => {
            let args_value = args
                .iter()
                .map(|a| interp_atom(a, &env, store))
                .collect::<Vec<Value<'a>>>();
            let result = builtin_call(prim, args_value);
            let mut new_env = env;
            new_env.insert(&bind, store.alloc(result));
            interp(body, new_env, store)
        }
        IR::LetVal(_label, var, val, body) => {
            let value = interp_atom(val, &env, store);
            let mut new_env = env;
            new_env.insert(var, store.alloc(value));
            interp(body, new_env, store)
        }
        IR::If(_label, test, then_, else_) => match interp_atom(test, &env, store) {
            Value::Bool(b) => {
                if b {
                    interp(then_, env, store)
                } else {
                    interp(else_, env, store)
                }
            }
            _ => {
                panic!("if: test should be a boolean")
            }
        },
        IR::App(_label, f, args, cont) => {
            let f_value = interp_atom(f, &env, store);
            let Value::Clo(vars, body, clo_env) = f_value else {
                panic!("application: not a function");
            };
            let mut new_env = clo_env;
            for (arg, var) in args.iter().zip(vars.iter()) {
                let val = interp_atom(arg, &env, store);
                new_env.insert(var, store.alloc(val));
            }
            let result = interp(body, new_env, store);
            apply_cont(cont, vec![result], env, store)
        }
        IR::Fix(_label, vars, vals, body) => {
            let mut new_env = env;
            let mut offset = store.get_count();
            for var in vars {
                new_env.insert(&var, store.alloc(Value::Bool(false)));
            }
            for val in vals {
                store.set_mem(offset, interp_atom(val, &new_env, store));
                offset += 1;
            }
            interp(body, new_env, store)
        }
        IR::AppCont(_label, cont, args) => match cont {
            Cont::Return => interp_atom(&args[0], &env, store),
            Cont::Named(name) => {
                let values = args
                    .iter()
                    .map(|arg| interp_atom(arg, &env, store))
                    .collect();
                apply_cont_by_name(&name, values, env, store)
            }
        },
    }
}
