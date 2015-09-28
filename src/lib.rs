extern crate rand;
extern crate time;

pub mod macaroni {
    use rand;
    use time;
    use std::collections::HashMap;
    use std::io;
    use std::rc::Rc;

    const DIGITS: &'static [u8] = b"0123456789";
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
        Op { func: Rc<Fn(&mut Macaroni, &[Variable]) -> Option<Variable>>, arity: usize },
        Label(String),
        Goto { label: String, noreturn: bool },
        Set(String)
    }

    pub struct Macaroni {
        vars: HashMap<String, Val>,
        program: Vec<Token>
    }

    impl Macaroni {
        pub fn new() -> Macaroni {
            Macaroni {
                vars: HashMap::<String, Val>::new(),
                program: vec![]
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
                    '"' | '/' | '\\' | ':' => {
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
                    '/' => {
                        Token::Label(t[1..t.len()].to_string())
                    },
                    '\\' => {
                        if t.starts_with("\\_") {
                            Token::Goto {
                                label: t[2..t.len()].to_string(),
                                noreturn: true
                            }
                        } else {
                            Token::Goto {
                                label: t[1..t.len()].to_string(),
                                noreturn: false
                            }
                        }
                    },
                    ':' => {
                        Token::Set(t[1..t.len()].to_string())
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
                        "concat" => Token::Op {
                            func: Rc::new(Macaroni::concat), arity: 2
                        },
                        "map" => Token::Op {
                            func: Rc::new(Macaroni::map), arity: 2
                        },
                        "length" => Token::Op {
                            func: Rc::new(Macaroni::length), arity: 1
                        },
                        "slice" => Token::Op {
                            func: Rc::new(Macaroni::slice), arity: 4
                        },
                        "wrap" => Token::Op {
                            func: Rc::new(Macaroni::wrap), arity: 1
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
                        _ => Token::Var(Variable::by_name(t.clone()))
                    } }
                } }).collect::<Vec<Token>>()
        }

        fn run_tokens(&mut self, from: usize) -> Option<Val> {
            let mut label_addrs: HashMap<String, usize> = HashMap::new();
            let mut i: usize = from;
            let mut to_set: Option<String> = None;
            let mut last_val: Option<Val> = None;
            let mut call_stack: Vec<usize> = Vec::new();
            loop {
                let t = if let Some(x) = self.program.get(i) {
                    x.clone()
                } else { break };
                match t {
                    Token::Set(var_name) => {
                        to_set = Some(var_name);
                        i += 1;
                        continue;
                    },
                    Token::Op { .. } => {
                        last_val = self.execute_op(&mut i)
                            .map(|x| x.val);
                    },
                    Token::Label(label) => {
                        label_addrs.entry(label).or_insert(i);
                        i += 1;
                    },
                    Token::Goto { label, noreturn } => {
                        if label == "" {
                            if let Some(x) = call_stack.pop() {
                                i = x;
                            } else { return last_val; }
                        } else {
                            if !noreturn { call_stack.push(i + 1); }
                            i = *label_addrs.entry(label.clone())
                                .or_insert_with(||
                                    self.find_label(&label, i)
                                        .expect(&format!("{:#08x}: goto to \
                                                         nonexistent label {:?}",
                                                         i, label)[..])
                                );
                        }
                    },
                    Token::Var(ref v) => {
                        last_val = Some(self.uv(v).val);
                        i += 1;
                    }
                }
                if let Some(ref var_name) = to_set {
                    self.vars.insert(var_name.clone(), last_val.expect(
                        &format!("{:08x}: cannot set to null", i)));
                    last_val = None;
                }
                to_set = None;
            }
            last_val
        }

        fn execute_op(&mut self, i: &mut usize) -> Option<Variable> {
            let (func, arity) = match self.program.get(*i).unwrap().clone() {
                Token::Op { func, arity } => (func, arity),
                _ => unreachable!()
            };
            *i += 1;
            let mut args: Vec<Variable> = Vec::with_capacity(arity);
            while args.len() < arity {
                match self.program.get(*i).expect(
                        &format!("{:#08x}: expected operator arugment, found \
                                 end of program", i)).clone() {
                    Token::Var(ref v) => {
                        args.push(self.uv(v));
                        *i += 1;
                    },
                    Token::Op { .. } => {
                        match self.execute_op(i) {
                            Some(v) => args.push(v),
                            None => panic!("{:#08x}: cannot pass null to \
                                           operator", i)
                        }
                    }
                    _ => panic!("{:#08x}: cannot pass non-variable to \
                                operator", i)
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

        /// Searches the program for the specified label. Returns the position
        /// of the label in the program, or `None` if it was not found.
        ///
        /// `index` merely specifies a starting point. The entire `program`
        /// is always searched, starting at `index` and wrapping around
        /// if necessary.
        ///
        /// # Examples
        ///
        /// ```
        /// use Token::*;
        ///
        /// let print_func = Box::new(|s| println!("{}", s));
        /// let program = vec![
        ///     Label(String::from("loop")),
        ///     Op({ func: print, arity: 1 }),
        ///     Variable(String::from("Hello, world!")),
        ///     Goto(String::from("loop"))
        /// ];
        /// assert_eq!(find_label(program, String::from("loop"), 0), Some(0));
        /// assert_eq!(find_label(program, String::from("cleanup"), 0), None);
        /// ```
        fn find_label(&self, desired_label: &str,
                index: usize) -> Option<usize> {
            let enumerated = self.program.iter().enumerate();
            for (i, token) in enumerated.clone()
                                        .skip(index)
                                        .chain(enumerated.take(index)) {
                if let Token::Label(ref label) = *token {
                    if label == desired_label {
                        return Some(i);
                    }
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

        fn map(&mut self, args: &[Variable]) -> Option<Variable> {
            let arr = match args[0].val {
                Val::Arr(ref a) => a,
                Val::Num(_) => panic!("map called with Num")
            }.clone();
            let lbl = match args[1].var {
                Some(ref x) => x,
                None => panic!("map called without label")
            };
            Some(Variable::new_arr(arr.into_iter().map(|x| {
                self.vars.insert("_".to_string(), x.clone());
                x
            }).collect()))
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

        fn wrap(&mut self, args: &[Variable]) -> Option<Variable> {
            let ref x = args[0];
            Some(Variable::new_arr(vec![x.val.clone()]))
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
