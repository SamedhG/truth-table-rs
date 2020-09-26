use rustyline::Editor;
use sexp::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Eq, Clone)]
enum LogicExp {
    Variable(String),
    Not(Box<LogicExp>),
    And(Box<LogicExp>, Box<LogicExp>),
    Or(Box<LogicExp>, Box<LogicExp>),
    Implies(Box<LogicExp>, Box<LogicExp>),
    Iff(Box<LogicExp>, Box<LogicExp>),
}

#[derive(Debug)]
enum Error {
    ParseError,
}

impl LogicExp {
    fn parse(sexp: Sexp) -> Result<Self, Error> {
        let not_sym = Sexp::Atom(Atom::S("-".to_string()));

        match sexp {
            Sexp::Atom(Atom::S(s)) => Ok(LogicExp::Variable(s)),
            Sexp::List(arr) => match arr[..] {
                [ref not, ref exp] if not.clone() == not_sym => {
                    Ok(LogicExp::Not(Box::new(LogicExp::parse(exp.clone())?)))
                }
                [ref exp0, ref sym, ref exp1] => {
                    if let Sexp::Atom(Atom::S(s)) = sym {
                        let e0 = Box::new(LogicExp::parse(exp0.clone())?);
                        let e1 = Box::new(LogicExp::parse(exp1.clone())?);
                        match &s[..] {
                            "*" => Ok(LogicExp::And(e0, e1)),
                            "+" => Ok(LogicExp::Or(e0, e1)),
                            "=>" => Ok(LogicExp::Implies(e0, e1)),
                            "<=>" => Ok(LogicExp::Iff(e0, e1)),
                            _ => Err(Error::ParseError),
                        }
                    } else {
                        Err(Error::ParseError)
                    }
                }
                _ => Err(Error::ParseError),
            },
            _ => Err(Error::ParseError),
        }
    }

    fn solve(&self, map: &HashMap<String, bool>) -> bool {
        match self {
            LogicExp::Variable(s) => map[s],
            LogicExp::And(e0, e1) => e0.solve(map) & e1.solve(map),
            LogicExp::Or(e0, e1) => e0.solve(map) | e1.solve(map),
            LogicExp::Not(e) => !e.solve(map),
            LogicExp::Implies(e0, e1) => (!e0.solve(map)) | e1.solve(map),
            LogicExp::Iff(e0, e1) => {
                ((!e0.solve(map)) | e1.solve(map)) & ((!e1.solve(map)) | e0.solve(map))
            }
        }
    }

    fn print_latex(&self) -> String {
        match self {
            LogicExp::Variable(s) => s.clone(),
            LogicExp::Not(e) => format!("\\neg {}", e.print_latex()),
            LogicExp::And(e0, e1) => format!("({} \\wedge {})", e0.print_latex(), e1.print_latex()),
            LogicExp::Or(e0, e1) => format!("({} \\vee {})", e0.print_latex(), e1.print_latex()),
            LogicExp::Implies(e0, e1) => {
                format!("({} \\rightarrow {})", e0.print_latex(), e1.print_latex())
            }
            LogicExp::Iff(e0, e1) => format!("({} \\iff {})", e0.print_latex(), e1.print_latex()),
        }
    }

    fn find_vars(&self) -> HashSet<String> {
        match self {
            LogicExp::Variable(s) => {
                let mut set = HashSet::new();
                set.insert(s.clone());
                set
            }
            LogicExp::Not(e) => e.find_vars(),
            LogicExp::And(e0, e1)
            | LogicExp::Or(e0, e1)
            | LogicExp::Implies(e0, e1)
            | LogicExp::Iff(e0, e1) => e0.find_vars().union(&e1.find_vars()).cloned().collect(),
        }
    }

    fn simple_table(&self) -> String {
        let mut vars: Vec<String> = self.find_vars().into_iter().collect();
        // So that the order is consistent
        vars.sort();
        let num_vars = vars.len();
        let num_iterations = (num_vars as f64).exp2() as usize;
        let mut s = String::new();

        // Generate the headers
        let mut fmt_s = String::from("|L|");
        let mut header_s = String::new();
        for var in &vars {
            fmt_s.push_str("L|");
            header_s.push_str(&format!(" {} &", var));
        }
        header_s.push_str(&format!("{} \\\\\n\\hline\n", self.print_latex()));

        for i in 0..num_iterations {
            let mut map = HashMap::new();
            let mut num = i;
            for var in &vars {
                let condition = num % 2 == 0;
                map.insert(var.clone(), condition);
                s.push_str(if condition { " T &" } else { " F &" });
                num /= 2;
            }
            let solved = self.solve(&map);
            s.push_str(if solved { " T \\\\\n" } else { " F \\\\\n" });
        }
        format!(
            "\\begin{{tabular}}{{{}}}\n{}{}\\end{{tabular}}",
            fmt_s, header_s, s
        )
    }

    fn get_steps(&self) -> Vec<Self> {
        let mut prev = match self {
            LogicExp::Variable(_) => Vec::new(),
            LogicExp::Not(e) => e.get_steps(),
            LogicExp::And(e0, e1)
            | LogicExp::Or(e0, e1)
            | LogicExp::Implies(e0, e1)
            | LogicExp::Iff(e0, e1) => {
                let mut v0 = e0.get_steps();
                let v1 = e1.get_steps();
                v1.into_iter().for_each(|x| {
                    if !v0.contains(&x) {
                        v0.push(x);
                    }
                });
                v0
            }
        };
        prev.push(self.clone());
        prev
    }

    fn steps_table(&self) -> String {
        let steps = self.get_steps();
        let mut vars: Vec<String> = self.find_vars().into_iter().collect();
        vars.sort();

        let num_vars = vars.len();
        let num_iterations = (num_vars as f64).exp2() as usize;
        let mut s = String::new();

        // Generate the headers
        let mut fmt_s = String::from("|");
        let mut header_s = String::from("\\hline\n");
        for (i, step) in steps.iter().enumerate() {
            fmt_s.push_str("c|");
            header_s.push_str(&step.print_latex());
            header_s.push_str(if i == (steps.len() - 1) {
                "\\\\\n\\hline\n"
            } else {
                "&"
            });
        }

        for i in 0..num_iterations {
            let mut map = HashMap::new();
            let mut num = i;
            for var in &vars {
                let condition = num % 2 == 0;
                map.insert(var.clone(), condition);
                num /= 2;
            }
            for (i, step) in steps.iter().enumerate() {
                let solved = step.solve(&map);
                s.push_str(if solved { " T " } else { " F " });
                s.push_str(if i == (steps.len() - 1) {
                    "\\\\\n\\hline\n"
                } else {
                    "&"
                });
            }
        }
        format!(
            "\\begin{{tabular}}{{{}}}\n{}{}\\end{{tabular}}",
            fmt_s, header_s, s
        )
    }
}

fn main() {
    let no_steps = std::env::args().last() == Some(String::from("--no-steps"));
    let mut rl = Editor::<()>::new();
    loop {
        let line = rl.readline(">> ");
        if line.is_err() {
            break;
        };
        let line = line.unwrap();
        let sexp = sexp::parse(&line);
        let sexp = match sexp {
            Ok(s) => s,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        let lexp = LogicExp::parse(sexp);
        if lexp.is_err() {
            println!("can't parse line");
            continue;
        }
        let lexp = lexp.unwrap();
        if no_steps {
            println!("{}", lexp.simple_table());
        } else {
            println!("{}", lexp.steps_table());
        }
    }
}
