extern crate rand;
extern crate time;

pub mod macaroni {
    use rand;
    use time;
    use std::collections::HashMap;
    use std::io;
    use std::rc::Rc;

    const DIGITS: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const EPSILON: f64 = 0.000001;
    const PRECISION: i32 = 10;

    #[derive(Clone)]
    pub enum Val {
        Num(f64),
        Arr(Vec<Val>)
    }

    use std::fmt;
    impl fmt::Debug for Val {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &Val::Num(n) => write!(f, "{}", n),
                &Val::Arr(ref a) => write!(f, "{:?}", a)
            }
        }
    }

    #[derive(Clone)]
    struct Variable {
        val: Val,
        var: Option<String>
    }

    impl Variable {
        fn by_name(name: String) -> Variable {
            Variable { val: Val::Num(0f64), var: Some(name) }
        }
        fn new_num(n: f64) -> Variable {
            Variable { val: Val::Num(n), var: None }
        }
        fn new_arr(a: Vec<Val>) -> Variable {
            Variable { val: Val::Arr(a), var: None }
        }
    }

    #[derive(Clone)]
    enum Token {
        Var(Variable),
        Op {
            func: Rc<Fn(&mut Macaroni, &[Variable]) -> Option<Variable>>,
            arity: usize
        },
        Label
    }

    struct State {
        i: usize,
        call_stack: Vec<usize>
    }

    pub struct Macaroni {
        vars: HashMap<String, Val>,
        program: Vec<Token>,
        states: Vec<State>
    }

    impl Macaroni {
        pub fn new() -> Macaroni {
            Macaroni {
                vars: HashMap::<String, Val>::new(),
                program: vec![], states: vec![]
            }
        }

        pub fn run(&mut self, code: String) -> Option<Val> {
            let tokens = self.tokenize(code);
            self.program = tokens;
            self.run_tokens(0)
        }

        fn tokenize(&self, code: String) -> Vec<Token> {
            let mut tokens = Vec::<String>::new();
            let mut token = String::new();
            for ch in code.chars() {
                if token.starts_with("\"") {
                    token.push(ch);
                    if ch == '"' {
                        tokens.push(token);
                        token = String::new();
                    }
                } else { match ch {
                    'a'...'z' | 'A'...'Z' | '0'...'9' | '_' | '-' => {
                        token.push(ch);
                    },
                    ' ' | '\n' | '\t' => {
                        if !token.is_empty() {
                            tokens.push(token);
                            token = String::new();
                        }
                    },
                    '"' | '/' | '\\' => {
                        if token.is_empty() {
                            token.push(ch);
                        } else {
                            panic!("unexpected {}", token);
                        }
                    },
                    _ => { panic!("unrecognized char {}", ch); }
                } }
            }
            if !token.is_empty() { tokens.push(token); }
            tokens.iter().map(|t|
                if t.chars().all(|ch| ch.is_digit(10) || ch == '-') {
                    Token::Var(Variable::new_num(t.parse::<f64>().unwrap()))
                } else { match t.chars().next().unwrap() {
                    '"' => {
                        Token::Var(Variable::new_arr(Macaroni::string_to_arr(
                            &t[1..t.len() - 1].to_string())))
                    },
                    _ => { match &t[..] {
                        "add" => Token::Op {
                            func: Rc::new(Macaroni::add), arity: 2
                        },
                        "multiply" => Token::Op {
                            func: Rc::new(Macaroni::multiply), arity: 2
                        },
                        "floor" => Token::Op {
                            func: Rc::new(Macaroni::floor), arity: 1
                        },
                        "pow" => Token::Op {
                            func: Rc::new(Macaroni::pow), arity: 2
                        },
                        "tobase" => Token::Op {
                            func: Rc::new(Macaroni::tobase), arity: 2
                        },
                        "sort" => Token::Op {
                            func: Rc::new(Macaroni::sort), arity: 2
                        },
                        "concat" => Token::Op {
                            func: Rc::new(Macaroni::concat), arity: 2
                        },
                        "each" => Token::Op {
                            func: Rc::new(Macaroni::each), arity: 2
                        },
                        "map" => Token::Op {
                            func: Rc::new(Macaroni::map), arity: 2
                        },
                        "index" => Token::Op {
                            func: Rc::new(Macaroni::index), arity: 2
                        },
                        "length" => Token::Op {
                            func: Rc::new(Macaroni::length), arity: 1
                        },
                        "transpose" => Token::Op {
                            func: Rc::new(Macaroni::transpose), arity: 1
                        },
                        "flatten" => Token::Op {
                            func: Rc::new(Macaroni::flatten), arity: 2
                        },
                        "frombase" => Token::Op {
                            func: Rc::new(Macaroni::frombase), arity: 2
                        },
                        "slice" => Token::Op {
                            func: Rc::new(Macaroni::slice), arity: 4
                        },
                        "wrap" => Token::Op {
                            func: Rc::new(Macaroni::wrap), arity: 1
                        },
                        "unwrap" => Token::Op {
                            func: Rc::new(Macaroni::unwrap), arity: 1
                        },
                        "print" => Token::Op {
                            func: Rc::new(Macaroni::print), arity: 1
                        },
                        "read" => Token::Op {
                            func: Rc::new(Macaroni::read), arity: 0
                        },
                        "rand" => Token::Op {
                            func: Rc::new(Macaroni::rand), arity: 0
                        },
                        "time" => Token::Op {
                            func: Rc::new(Macaroni::time), arity: 0
                        },
                        "set" => Token::Op {
                            func: Rc::new(Macaroni::set), arity: 2
                        },
                        "label" => Token::Label,
                        "goto" => Token::Op {
                            func: Rc::new(Macaroni::goto), arity: 1
                        },
                        "return" => Token::Op {
                            func: Rc::new(Macaroni::return_), arity: 0
                        },
                        _ => Token::Var(Variable::by_name(t.clone()))
                    } }
                } }).collect::<Vec<Token>>()
        }

        fn run_tokens(&mut self, from: usize) -> Option<Val> {
            self.states.insert(0, State { i: from, call_stack: vec![] });
            let mut last_val: Option<Val> = None;
            loop {
                let t = if let Some(x) = self.program.get(self.states[0].i) {
                    x.clone()
                } else { break };
                match t {
                    Token::Op { .. } => {
                        last_val = self.execute_op().map(|x| x.val);
                    },
                    Token::Var(ref v) => {
                        last_val = Some(self.uv(v).val);
                        self.states[0].i += 1;
                    },
                    Token::Label => {
                        self.states[0].i += 2;
                    }
                }
            }
            self.states.remove(0);
            last_val
        }

        fn execute_op(&mut self) -> Option<Variable> {
            let (func, arity) = match self.program.get(self.states[0].i)
                    .unwrap().clone() {
                Token::Op { func, arity } => (func, arity),
                _ => unreachable!()
            };
            self.states[0].i += 1;
            let mut args: Vec<Variable> = Vec::with_capacity(arity);
            while args.len() < arity {
                match self.program.get(self.states[0].i).expect(
                        &format!("{:#08x}: expected operator argument, found \
                                 end of program", self.states[0].i)).clone() {
                    Token::Var(ref v) => {
                        args.push(self.uv(v));
                        self.states[0].i += 1;
                    },
                    Token::Op { .. } => {
                        match self.execute_op() {
                            Some(v) => args.push(v),
                            None => panic!("{:#08x}: cannot pass null to \
                                           operator", self.states[0].i)
                        }
                    },
                    Token::Label => panic!("{:#08x}: cannot pass label to \
                                           operator", 1)
                }
            }
            func(self, &args[..])
        }

        /// Updates the `val` part of a variable, when `var` is Some().
        fn uv(&self, v: &Variable) -> Variable {
            if let Some(ref var_name) = v.var {
                Variable {
                    val: self.vars.get(var_name)
                        .map_or(Val::Num(0f64), |x| x.clone()),
                    var: v.var.clone()
                }
            } else { v.clone() }
        }

        fn find_label(&self, desired_label: &str) -> Option<usize> {
            for i in 0..self.program.len() - 1 {
                match self.program[i] {
                    Token::Label => match self.program[i + 1] {
                        Token::Var(ref v) => match v.var {
                            Some(ref name) => if name == desired_label {
                                return Some(i);
                            },
                            _ => ()
                        },
                        _ => ()
                    },
                    _ => ()
                }
            }
            None
        }

        fn add(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            } + match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }))
        }

        fn multiply(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("multiply called with Arr")
            } * match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("multiply called with Arr")
            }))
        }

        fn floor(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("floor called with Arr")
            }.floor()))
        }

        fn pow(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("pow called with Arr")
            }.powf(match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("pow called with Arr")
            })))
        }

        fn tobase(&mut self, args: &[Variable]) -> Option<Variable> {
            match args[0].val { Val::Num(n) => {
                match args[1].val { Val::Num(m) => {
                    let (base, mut ipart, mut fpart) =
                        (m as i64, n.floor() as i64, n - n.floor());
                    let neg = ipart < 0;

                    // convert integer part
                    let mut nb = String::new();
                    while ipart != 0 {
                        nb.insert(0, DIGITS[(ipart % base) as usize] as char);
                        ipart /= base;
                    }

                    // handle negatives and zero (we must do this
                    // before converting float part)
                    if neg { nb.insert(0, '-'); }
                    if nb.is_empty() { nb.push('0'); }

                    // convert float part
                    if fpart > EPSILON {
                        nb.push('.');
                        for _ in 0..PRECISION {
                            if fpart <= EPSILON { break; }
                            fpart *= base as f64;
                            let digit = fpart.floor();
                            nb.push(DIGITS[digit as usize] as char);
                            fpart -= digit;
                        }
                    }

                    Some(Variable::new_arr(Macaroni::string_to_arr(&nb)))
                }, Val::Arr(_) => panic!("tobase called with Arr") }
            }, Val::Arr(_) => panic!("tobase called with Arr") }
        }

        fn sort(&mut self, args: &[Variable]) -> Option<Variable> {
            let mut arr = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("map called with Num")
            }.clone();
            let lbl = match args[1].var {
                Some(ref x) => x,
                None => panic!("map called without label")
            };
            let lbl_idx = self.find_label(lbl).expect(&format!(""));
            arr.sort_by(|a, b| {
                self.vars.insert("_".to_string(),
                    Val::Arr(vec![a.clone(), b.clone()]));
                self.run_tokens(lbl_idx);
                // there's no way to unset a variable, so we can unwrap
                match self.vars.get("_").unwrap() {
                    &Val::Num(n) => n.partial_cmp(&0f64).unwrap(),
                    &Val::Arr(_) => panic!("sort predicate returned Arr")
                }
            });
            Some(Variable::new_arr(arr))
        }

        fn concat(&mut self, args: &[Variable]) -> Option<Variable> {
            let mut arr = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("concat called with Num")
            }.clone();
            arr.extend(match args[1].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("concat called with Num")
            }.clone().into_iter());
            Some(Variable::new_arr(arr))
        }

        fn each(&mut self, args: &[Variable]) -> Option<Variable> {
            let a = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("each called with Num")
            }.clone();
            let (neg, n) = match args[1].val {
                Val::Num(n) => (n < 0f64, if n < 0f64 { -n } else { n } as usize),
                Val::Arr(_) => panic!("each called with Arr")
            };
            let mut arr = Vec::<Val>::new();
            if neg {
                // full subarrays
                for i in 0..a.len() / n {
                    arr.push(Val::Arr(Vec::from(&a[i * n..(i + 1) * n])));
                }
                // perhaps one partial subarray
                if a.len() % n != 0 {
                    arr.push(Val::Arr(Vec::from(&a[a.len() / n * n..a.len()])));
                }
            } else {
                if a.len() >= n {
                    for i in 0..a.len() - n + 1 {
                        arr.push(Val::Arr(Vec::from(&a[i..i + n])));
                    }
                }
            }
            Some(Variable::new_arr(arr))
        }

        fn map(&mut self, args: &[Variable]) -> Option<Variable> {
            let arr = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("map called with Num")
            }.clone();
            let lbl = match args[1].var {
                Some(ref x) => x,
                None => panic!("map called without label")
            };
            let lbl_idx = self.find_label(lbl).expect(&format!(""));
            Some(Variable::new_arr(arr.into_iter().map(|x| {
                self.vars.insert("_".to_string(), x.clone());
                self.run_tokens(lbl_idx);
                self.vars.get("_").unwrap().clone()
            }).collect()))
        }

        fn index(&mut self, args: &[Variable]) -> Option<Variable> {
            let arr = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("index called with Num")
            }.clone();
            let lbl = match args[1].var {
                Some(ref x) => x,
                None => panic!("index called without label")
            };
            let lbl_idx = self.find_label(lbl).expect(&format!(""));
            Some(Variable::new_arr(arr.into_iter().enumerate().filter(|&(_, ref x)| {
                self.vars.insert("_".to_string(), x.clone());
                self.run_tokens(lbl_idx);
                match self.vars.get("_").unwrap() {
                    &Val::Arr(ref a) => !a.is_empty(),
                    &Val::Num(n) => n != 0f64
                }
            }).map(|(i, _)| Val::Num(i as f64)).collect()))
        }

        fn slice(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_arr(match args[0].val {
                Val::Arr(ref a) => {
                    let (step, rev) = match args[3].val {
                        Val::Num(n) => {
                            if n > 0f64 {
                                (n as usize, false)
                            } else {
                                (-n as usize, true)
                            }
                        },
                        Val::Arr(_) => panic!("slice called with Arr")
                    };
                    let (mut idx, to) = (
                        match args[if rev { 2 } else { 1 }].val {
                            Val::Num(n) => n as usize,
                            Val::Arr(_) => panic!("slice called with Arr")
                        },
                        match args[if rev { 1 } else { 2 }].val {
                            Val::Num(n) => n as usize,
                            Val::Arr(_) => panic!("slice called with Arr")
                        }
                    );
                    if rev {
                        if idx > a.len() { idx = a.len(); }
                        else if idx > 0 { idx -= 1; }
                        else { return Some(Variable::new_arr(vec![])); }
                    }
                    let mut new_arr = Vec::<Val>::new();
                    while if rev { idx >= to } else { idx < to } {
                        if idx < a.len() {
                            new_arr.push(a[idx].clone());
                        } else {
                            if !rev { break; }
                        }
                        if rev {
                            if step > idx { break; }
                            idx -= step;
                        } else { idx += step; }
                    }
                    new_arr
                }
                Val::Num(_) => panic!("slice called with Num")
            }))
        }

        fn length(&mut self, args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Arr(ref a) => a.len() as f64,
                Val::Num(_) => panic!("length called with Num")
            }))
        }

        fn transpose(&mut self, args: &[Variable]) -> Option<Variable> {
            let arr = match args[0].val {
                Val::Arr(ref a) => a.clone(),
                Val::Num(_) => panic!("transpose called with Num")
            };
            if let Some(max_len) = arr.iter().map(|x|
                match x {
                    &Val::Arr(ref a) => a.len(),
                    &Val::Num(_) => panic!("transpose called with non-2D Arr")
                }
            ).max() {
                Some(Variable::new_arr((0..max_len).map(|i|
                    Val::Arr(arr.iter().filter_map(|x|
                        match x {
                            &Val::Arr(ref a) => a,
                            &Val::Num(_) => unreachable!()
                        }.get(i).map(|val| val.clone())
                    ).collect())
                ).collect()))
            } else {
                Some(Variable::new_arr(vec![]))
            }
        }

        fn flatten(&mut self, args: &[Variable]) -> Option<Variable> {
            let mut arr = match args[0].val {
                Val::Arr(ref a) => a.clone(),
                Val::Num(_) => panic!("flatten called with Num")
            };
            let n = match args[1].val {
                Val::Num(n) => n as usize,
                Val::Arr(_) => panic!("flatten called with Arr")
            };
            for _ in 0..if n == 0 { usize::max_value() } else { n } {
                let mut tmp = Vec::<Val>::new();
                let mut found_arr = false;
                for val in arr {
                    match val {
                        Val::Arr(a) => { found_arr = true; tmp.extend(a); },
                        Val::Num(n) => tmp.push(Val::Num(n))
                    }
                }
                arr = tmp;
                if !found_arr { break }
            }
            Some(Variable::new_arr(arr))
        }

        fn frombase(&mut self, args: &[Variable]) -> Option<Variable> {
            match args[0].val { Val::Arr(ref s) => {
                match args[1].val { Val::Num(n) => {
                    let (base, mut nb) = (n as i64, Macaroni::arr_to_string(&s));

                    // handle negatives and decimals
                    let neg = nb.starts_with("-");
                    if neg { nb.remove(0); }

                    let sub_pos = if let Some(dot_pos) = nb.find('.') {
                        nb.remove(dot_pos);
                        dot_pos - 1
                    } else {
                        nb.len() - 1
                    };

                    // convert cleaned-up string
                    let mut n = 0f64;
                    for i in (0..nb.len()).rev() {
                        let c = nb.chars().nth(i).unwrap().to_uppercase()
                            .next().unwrap() as u8;
                        let digit = DIGITS.iter().position(|d| *d == c)
                            .expect(&format!("unrecognized digit {}", c)) as f64;
                        n += digit * base.pow((sub_pos - i) as u32) as f64;
                    }

                    Some(Variable::new_num(n))
                }, Val::Arr(_) => panic!("frombase called with Arr") }
            }, Val::Num(_) => panic!("frombase called with Num") }
        }

        fn wrap(&mut self, args: &[Variable]) -> Option<Variable> {
            let ref x = args[0];
            Some(Variable::new_arr(vec![x.val.clone()]))
        }

        fn unwrap(&mut self, args: &[Variable]) -> Option<Variable> {
            match args[0].val {
                Val::Arr(ref a) => {
                    if a.len() == 1 {
                        Some(Variable { val: a[0].clone(), var: None })
                    } else {
                        panic!("unwrap called with Arr of length != 1")
                    }
                },
                Val::Num(_) => panic!("unwrap called with Num")
            }
        }

        fn print(&mut self, args: &[Variable]) -> Option<Variable> {
            let ref x = args[0];
            print!("{}", match x.val {
                Val::Arr(ref s) => Macaroni::arr_to_string(s),
                Val::Num(_) => panic!("print called with Num")
            });
            None
        }

        fn read(&mut self, _: &[Variable]) -> Option<Variable> {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            Some(Variable::new_arr(Macaroni::string_to_arr(&line)))
        }

        fn rand(&mut self, _: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(rand::random()))
        }

        fn time(&mut self, _: &[Variable]) -> Option<Variable> {
            let t = time::get_time();
            Some(Variable::new_num((t.sec as f64) + (t.nsec as f64) /
                                                     1000000000f64))
        }

        fn set(&mut self, args: &[Variable]) -> Option<Variable> {
            self.vars.insert(args[0].clone().var.expect("cannot set a literal"),
                args[1].clone().val);
            Some(Variable {
                val: args[1].clone().val, var: args[0].clone().var
            })
        }

        fn goto(&mut self, args: &[Variable]) -> Option<Variable> {
            match args[0].var {
                Some(ref label) => {
                    let old_i = self.states[0].i;
                    self.states[0].call_stack.push(old_i);
                    self.states[0].i = self.find_label(label)
                        .expect(&format!("goto unknown label"));
                },
                None => panic!("cannot goto literal")
            }
            None
        }

        fn return_(&mut self, _: &[Variable]) -> Option<Variable> {
            self.states[0].i = match self.states[0].call_stack.pop() {
                Some(i) => i,
                None => self.program.len()
            };
            None
        }

        fn arr_to_string(arr: &Vec<Val>) -> String {
            arr.iter().map(|x| match x {
                &Val::Num(n) => n,
                &Val::Arr(_) => panic!("arr_to_string called with non-string")
            } as u8 as char).collect::<String>()
        }

        fn string_to_arr(str: &String) -> Vec<Val> {
            str.chars().map(|c| Val::Num(c as u8 as f64)).collect()
        }
    }
}
