extern crate rand;

pub mod macaroni {
    use rand;
    use std::io;

    const DIGITS: &'static [u8] = b"0123456789";
    const EPSILON: f64 = 0.000001;
    const PRECISION: i32 = 10;

    #[derive(Clone)]
    enum Val {
        Num(f64),
        Arr(Vec<Val>)
    }

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

    enum Token {
        Var(Variable),
        Op { func: Box<Fn(&[&Variable]) -> Option<Variable>>, arity: usize },
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
            self.run_tokens(Macaroni::tokenize(code));
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
                            func: Box::new(Macaroni::add), arity: 2
                        },
                        "multiply" => Token::Op {
                            func: Box::new(Macaroni::multiply), arity: 2
                        },
                        "floor" => Token::Op {
                            func: Box::new(Macaroni::floor), arity: 1
                        },
                        "pow" => Token::Op {
                            func: Box::new(Macaroni::pow), arity: 2
                        },
                        "tobase" => Token::Op {
                            func: Box::new(Macaroni::tobase), arity: 2
                        },
                        "wrap" => Token::Op {
                            func: Box::new(Macaroni::wrap), arity: 1
                        },
                        "print" => Token::Op {
                            func: Box::new(Macaroni::print), arity: 1
                        },
                        "read" => Token::Op {
                            func: Box::new(Macaroni::read), arity: 0
                        },
                        "rand" => Token::Op {
                            func: Box::new(Macaroni::rand), arity: 0
                        },
                        _ => Token::Var(Variable::by_name(t.clone()))
                    } }
                } }).collect::<Vec<Token>>()
        }

        fn run_tokens(&self, tokens: Vec<Token>) {
            let mut i: usize = 0;
            while let Some(t) = tokens.get(i) {
                match t {
                    &Token::Op { ref func, ref arity } => {
                        self.execute_op(&tokens, &mut i);
                    },
                    &Token::Goto(ref lbl) => {
                        // TODO
                    },
                    _ => {}
                }
                i += 1;
            }
        }

        fn execute_op(&self, tokens: &Vec<Token>, i: &mut usize) -> Option<Variable> {
            if let Token::Op { ref func, ref arity } = tokens[*i] {
                let mut args: Vec<&Variable> = Vec::with_capacity(*arity);
                *i += 1;
                while args.len() < *arity {
                    match tokens[*i] {
                        Token::Var(ref v) => {
                            args.push(v);
                        },
                        Token::Op { ref func, ref arity } => {
                            match self.execute_op(tokens, i) {
                                Some(v) => args.push(v),
                                None => panic!("put something helpful here")
                            }
                        }
                        _ => panic!("put something helpful here")

                    }
                    *i += 1;
                }
                return func(&args[..]);
            }
            panic!("put something helpful here");
        }

        fn add(args: &[&Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            } + match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }))
        }

        fn multiply(args: &[&Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            } * match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }))
        }

        fn floor(args: &[&Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }.floor()))
        }

        fn pow(args: &[&Variable]) -> Option<Variable> {
            Some(Variable::new_num(match args[0].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            }.powf(match args[1].val {
                Val::Num(n) => n,
                Val::Arr(_) => panic!("add called with Arr")
            })))
        }

        fn tobase(args: &[&Variable]) -> Option<Variable> {
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

        fn wrap(args: &[&Variable]) -> Option<Variable> {
            let ref x = args[0];
            Some(Variable::new_arr(vec![x.val.clone()]))
        }

        fn print(args: &[&Variable]) -> Option<Variable> {
            let ref x = args[0];
            print!("{}", match x.val {
                Val::Arr(ref s) => Macaroni::arr_to_string(s),
                Val::Num(_) => panic!("print called with Num")
            });
            None
        }

        fn read(_: &[&Variable]) -> Option<Variable> {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            Some(Variable::new_arr(Macaroni::string_to_arr(&line)))
        }

        fn rand(_: &[&Variable]) -> Option<Variable> {
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
