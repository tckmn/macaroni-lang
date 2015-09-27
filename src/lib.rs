extern crate rand;

pub mod macaroni {
    use rand;
    use std::collections::HashMap;
    use std::io;
    use std::rc::Rc;

    const DIGITS: &'static [u8] = b"0123456789";
    const EPSILON: f64 = 0.000001;
    const PRECISION: i32 = 10;

    #[derive(Clone)]
    enum Val {
        Num(f64),
        Arr(Vec<Val>)
    }

    #[derive(Clone)]
    struct Variable {
        val: Val,
        var: Option<String>
    }

    impl Variable {
        fn by_name(name: String) -> Variable {
            Variable { val: Val::Num(0f64), var: Some(name) } // TODO
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
        Op { func: Rc<Fn(&[Variable]) -> Option<Variable>>, arity: usize },
        Label(String),
        Goto(String)
    }

    pub struct Macaroni {
        vars: Vec<Variable>
    }

    impl Macaroni {
        pub fn new() -> Macaroni {
            Macaroni { vars: Vec::<Variable>::new() }
        }

        pub fn run(&self, code: String) {
            self.run_tokens(&Macaroni::tokenize(code));
        }

        fn tokenize(code: String) -> Vec<Token> {
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
                    '/' => {
                        Token::Label(t[1..t.len()].to_string())
                    },
                    '\\' => {
                        Token::Goto(t[1..t.len()].to_string())
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
                        _ => Token::Var(Variable::by_name(t.clone()))
                    } }
                } }).collect::<Vec<Token>>()
        }

        fn run_tokens(&self, program: &[Token]) {
            let mut label_addrs: HashMap<String, usize> = HashMap::new();
            let mut i: usize = 0;
            while let Some(t) = program.get(i) {
                match t {
                    &Token::Op { .. } => {
                        self.execute_op(&program, &mut i);
                    },
                    &Token::Label(ref label) => {
                        label_addrs.entry(label.clone()).or_insert(i);
                        i += 1;
                    },
                    &Token::Goto(ref label) => {
                        i = label_addrs.entry(label.clone())
                                       .or_insert_with(|| self.find_label(program, label, i).expect(
                                            &format!("{:#08x}: is a GOTO to label {:?}, which does not exist", i, label)[..]))
                                       .clone();
                    },
                    _ => {
                        i += 1;
                    }
                }
            }
        }

        fn execute_op(&self, program: &[Token], i: &mut usize) -> Option<Variable> {
            let (func, arity) = match program.get(i.clone()) {
                Some(&Token::Op { ref func, arity }) => (func, arity),
                Some(_) => panic!(format!("Macaroni::execute_op() called on {:#08x}, but instruction is not OP", i)),
                None => panic!(format!("Macaroni::execute_op() called on out-of-bounds index {:#08x}", i))
            };
            *i += 1;
            let mut args: Vec<Variable> = Vec::with_capacity(arity);
            while args.len() < arity {
                match program.get(i.clone()) {
                    Some(&Token::Var(ref v)) => {
                        args.push(v.clone());
                        *i += 1;
                    },
                    Some(&Token::Op { .. }) => {
                        match self.execute_op(program, i) {
                            Some(v) => args.push(v),
                            None => panic!(format!("nested OP ending before {:#08x} returned nothing", i))
                        }
                    }
                    Some(_) => panic!(format!("{:#08x} is not valid as an OP argument; it should be a VAR or an OP", i)),
                    None => panic!(format!("{:#08x}: expected OP argument (VAR or OP); found end of program", i))
                }
            }
            func(&args[..])
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
        /// let program = vec![Label(String::from("loop")), Op({ func: print, arity: 1 }), Variable(String::from("Hello, world!")), Goto(String::from("loop"))];
        /// assert_eq!(find_label(program, String::from("loop"), 0), Some(0));
        /// assert_eq!(find_label(program, String::from("cleanup"), 0), None);
        /// ```
        fn find_label(&self, program: &[Token], desired_label: &str, index: usize) -> Option<usize> {
            let enumerated = program.iter().enumerate();
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

        fn add(args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            } + match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }))
        }

        fn multiply(args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            } * match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }))
        }

        fn floor(args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }.floor()))
        }

        fn pow(args: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }.powf(match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            })))
        }

        fn tobase(args: &[Variable]) -> Option<Variable> {
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

        fn wrap(args: &[Variable]) -> Option<Variable> {
            let ref x = args[0];
            Some(Variable::new_arr(vec![x.val.clone()]))
        }

        fn print(args: &[Variable]) -> Option<Variable> {
            let ref x = args[0];
            print!("{}", match x.val {
                Val::Arr(ref s) => Macaroni::arr_to_string(s),
                Val::Num(_) => panic!("print called with Num")
            });
            None
        }

        fn read(_: &[Variable]) -> Option<Variable> {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            Some(Variable::new_arr(Macaroni::string_to_arr(&line)))
        }

        fn rand(_: &[Variable]) -> Option<Variable> {
            Some(Variable::new_num(rand::random()))
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
