use std::collections::HashMap;
use std::io::Write;
use std::{env, fs, io, process, str};

// ---------- Logging ----------

fn log_info(msg: &str) {
    println!("info: {}", msg);
}

macro_rules! log_error {
    () => {
        println!("\x1b[1;31merro\x1b[0m");
    };
    ( $($arg:tt)* ) => {
        print!("\x1b[1;31merro\x1b[0m: ");
        println!($($arg)*);
        std::process::exit(1);
    };
}

// ---------- Stack ----------

#[derive(Debug)]
pub struct Stack<T> {
    lista: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { lista: vec![] }
    }

    pub fn push(&mut self, valor: T) {
        self.lista.push(valor);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.lista.pop()
    }

    pub fn len(&self) -> usize {
        self.lista.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lista.is_empty()
    }
}

// ---------- Tokenizacao ----------

#[derive(Debug)]
pub enum Token {
    // operadores
    Igual,
    ParenAbr,
    ParenFec,
    Interrogacao,
    Exclamacao,
    // funcoes builtin
    Mais,  // soma 2 numeros
    Print, // print
    Pop,   // exclui topo da stack
    Dup,   // duplica todo da stack
    Swap,  // muda os dois ultimos itens da stack
    // numeros e nomes
    Numero(i32),
    Simbolo(String),
}

fn tokenizar(entrada: &str) -> Vec<Token> {
    let mut tokens = vec![];

    let mut buffer = String::new();
    let mut enumero = false;

    for c in entrada.chars() {
        if !buffer.is_empty() {
            if enumero {
                match c {
                    c @ '0'..'9' => {
                        buffer.push(c);
                    }
                    c if c.is_alphabetic() => {
                        // println!("nao pode letra dps de numero: {:?}", c);
                        // process::exit(1);
                        log_error!("nao pode letra dps de numero: {:?}", c);
                    }
                    _ => {
                        tokens.push(Token::Numero(
                            buffer.parse().expect("nunca deveria dar erro"),
                        ));
                        enumero = false;
                        buffer.clear();
                    }
                }
            } else {
                match c {
                    c if (c.is_alphabetic() | c.is_digit(10)) => {
                        buffer.push(c);
                    }
                    _ => {
                        tokens.push(Token::Simbolo(String::from(&buffer)));
                        buffer.clear();
                    }
                }
            }
        } else {
            match c {
                '=' => tokens.push(Token::Igual),
                '(' => tokens.push(Token::ParenAbr),
                ')' => tokens.push(Token::ParenFec),
                '?' => tokens.push(Token::Interrogacao),
                '!' => tokens.push(Token::Exclamacao),
                '+' => tokens.push(Token::Mais),

                c if c.is_digit(10) => {
                    buffer.push(c);
                    enumero = true;
                }
                c if c.is_alphabetic() => {
                    buffer.push(c);
                    enumero = false;
                }
                c if c.is_whitespace() => (),
                outro => {
                    println!("nao sei: {:?}", outro);
                    process::exit(1);
                }
            }
        }
    }

    if !buffer.is_empty() {
        if enumero {
            tokens.push(Token::Numero(
                buffer.parse().expect("nao deveria dar errado"),
            ));
        } else {
            tokens.push(Token::Simbolo(String::from(&buffer)));
        }
    }

    // for palavra in entrada.split_whitespace() {
    //     match palavra.trim() {
    //         "=" => tokens.push(Token::Igual),
    //         "(" => tokens.push(Token::ParenAbr),
    //         ")" => tokens.push(Token::ParenFec),
    //         "?" => tokens.push(Token::Interrogacao),
    //         "!" => tokens.push(Token::Exclamacao),

    //         "+" => tokens.push(Token::Mais),
    //         "print" => tokens.push(Token::Print),
    //         "pop" => tokens.push(Token::Pop),
    //         "dup" => tokens.push(Token::Dup),
    //         "swap" => tokens.push(Token::Swap),

    //         outro => {
    //             if let Ok(num) = outro.parse::<i32>() {
    //                 tokens.push(Token::Numero(num));
    //             } else {
    //                 tokens.push(Token::Simbolo(String::from(outro)));
    //             }
    //         }
    //     }
    // }
    println!("tokens: {:?}", tokens);
    process::exit(1);
    tokens
}

// ---------- AST ? ----------

/*
    [
        (nome1, [ 1 2 func ?(1 +) ! ]),
        (nome2, [ 3 4 + print ])
    ]

*/

#[derive(Debug, Clone)]
pub enum ASTItem {
    Mais,
    Print,
    Numero(i32),
    FuncDef(Func),
    FuncCallNamed(String),
    FuncCallTop,
}

type Func = Vec<ASTItem>;

type AST = Vec<(String, Func)>;

fn gerar_ast_funcao(tokens: &Vec<Token>, i: &mut usize) -> Func {
    let mut funcao_atual: Func = vec![];
    let mut stack_funcoes: Stack<Func> = Stack::new();
    let mut criando_funcao = false;

    loop {
        if *i == tokens.len() {
            break;
        }
        let atual = &tokens[*i];
        *i += 1;
        match atual {
            Token::ParenAbr => {
                if criando_funcao {
                    criando_funcao = false;
                    stack_funcoes.push(funcao_atual.to_vec());
                    funcao_atual.clear();
                } else {
                    println!("parenteses sem ter interrogacao antes");
                    process::exit(1);
                }
            }
            Token::ParenFec => {
                if stack_funcoes.is_empty() {
                    *i -= 1;
                    break;
                } else {
                    let f = funcao_atual.to_vec();
                    funcao_atual = match stack_funcoes.pop() {
                        Some(f) => f,
                        None => {
                            println!("erro no parenfec");
                            process::exit(1);
                        }
                    };
                    funcao_atual.push(ASTItem::FuncDef(f));
                }
            }
            Token::Interrogacao => {
                criando_funcao = true;
            }
            Token::Exclamacao => {
                funcao_atual.push(ASTItem::FuncCallTop);
            }
            Token::Mais => {
                funcao_atual.push(ASTItem::Mais);
            }
            Token::Print => {
                funcao_atual.push(ASTItem::Print);
            }
            Token::Pop => todo!(),
            Token::Dup => todo!(),
            Token::Swap => todo!(),
            Token::Numero(n) => {
                funcao_atual.push(ASTItem::Numero(*n));
            }
            Token::Simbolo(nome) => {
                funcao_atual.push(ASTItem::FuncCallNamed(nome.to_string()));
            }
            Token::Igual => {
                println!("impossível ter igual dentro de uma funcao");
                process::exit(1);
            }
        }
    }

    funcao_atual
}

fn gerar_ast(tokens: Vec<Token>, funcao: bool) -> AST {
    let mut ast = vec![];
    let mut i: usize = 0;

    if funcao {
        ast.push((String::from("funcao"), gerar_ast_funcao(&tokens, &mut i)));
    } else {
        loop {
            if i == tokens.len() {
                break;
            }
            let Token::Simbolo(nome) = &tokens[i] else {
                println!("falta nome no começo de uma funcao");
                process::exit(1);
            };
            i += 1;
            let Token::Igual = &tokens[i] else {
                println!("falta um igual na definicao da funcao {nome}");
                process::exit(1);
            };
            i += 1;
            let Token::ParenAbr = &tokens[i] else {
                println!("falta um parenteses no comeco da funcao {nome}");
                process::exit(1);
            };
            i += 1;
            let func = gerar_ast_funcao(&tokens, &mut i);
            let Token::ParenFec = &tokens[i] else {
                println!("falta um parenteses no final da funcao {nome}");
                process::exit(1);
            };
            i += 1;
            ast.push((nome.to_string(), func));
        }
    }

    // println!("ast: {ast:?}");

    ast
}

fn tokenizar_e_gerar_ast(entrada: &str, funcao: bool) -> AST {
    gerar_ast(tokenizar(entrada), funcao)
}

// ---------- Interpretacao ----------

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Item {
    Numero(i32),
    Func(Vec<ASTItem>),
}

pub fn interpretar_func(estado: &mut PSFState, func: Func) {
    let mut stack_consumir = Stack::new();
    for item in func.iter().rev() {
        stack_consumir.push(item.clone());
    }
    let mut item: ASTItem;
    loop {
        if estado.stack.len() > 1000 {
            println!("stack muito grande, terminando programa");
            process::exit(1);
        }
        item = match stack_consumir.pop() {
            Some(i) => i,
            None => break,
        };
        match item {
            ASTItem::Numero(n) => {
                estado.stack.push(Item::Numero(n));
            }
            ASTItem::Mais => {
                let Some(Item::Numero(n1)) = estado.stack.pop() else {
                    println!("tipo do primeiro argumento nao e numero");
                    process::exit(1);
                };
                let Some(Item::Numero(n2)) = estado.stack.pop() else {
                    println!("tipo do segundo argumento nao e numero");
                    process::exit(1);
                };
                estado.stack.push(Item::Numero(n1 + n2));
            }
            ASTItem::Print => {
                let Some(item) = estado.stack.pop() else {
                    println!("stack vazia antes do print");
                    process::exit(1);
                };
                estado.stack.push(item.clone());
                match item {
                    Item::Numero(n) => {
                        println!("{n}");
                    }
                    // Item::Mais => todo!(),
                    // Item::Print => todo!(),
                    // Item::FuncDef(items) => todo!(),
                    // Item::FuncCallNamed(_) => todo!(),
                    // Item::FuncCallTop => todo!(),
                    outro => {
                        println!("impossivel: {:?}", outro);
                    }
                }
            }
            ASTItem::FuncDef(f) => {
                estado.stack.push(Item::Func(f));
            }
            ASTItem::FuncCallTop => {
                let Some(f) = estado.stack.pop() else {
                    println!("tentativa de chamar topo da funcao mas nao tem item");
                    process::exit(1);
                };
                let Item::Func(f) = f else {
                    println!("topo da funcao nao é funcao");
                    process::exit(1);
                };
                for i in f.iter().rev() {
                    stack_consumir.push(i.clone());
                }
            }
            ASTItem::FuncCallNamed(f) => {
                let Some(f) = estado.funcoes.get(&f) else {
                    println!("funcao `{:?}` nao existe", f);
                    process::exit(1);
                };
                for i in f.iter().rev() {
                    stack_consumir.push(i.clone());
                }
            }
        }
    }
}

// ---------- Estado ----------

#[derive(Debug)]
pub struct PSFState {
    #[allow(dead_code, unused)]
    stack: Stack<Item>,
    #[allow(dead_code, unused)]
    funcoes: HashMap<String, Func>,
}

impl PSFState {
    pub fn new() -> PSFState {
        PSFState {
            stack: Stack::new(),
            funcoes: HashMap::new(),
        }
    }

    pub fn clear_stack(&mut self) {
        self.stack.lista.clear();
    }

    #[allow(dead_code, unused)]
    pub fn load_ast(&mut self, ast: AST) {
        for (nome, funcao) in ast {
            self.funcoes.insert(nome, funcao);
        }
    }

    #[allow(dead_code, unused)]
    pub fn load_string(&mut self, entrada: &str) {
        self.load_ast(tokenizar_e_gerar_ast(entrada, false));
    }

    #[allow(dead_code, unused)]
    pub fn run_raw_string(&mut self, entrada: &str) {
        let itens = tokenizar_e_gerar_ast(entrada, true);
        let (_, funcao) = itens.get(0).unwrap();
        interpretar_func(self, funcao.to_vec());
    }

    pub fn run_function(&mut self, f: &str) {
        interpretar_func(
            self,
            self.funcoes.get(f).expect("funcao nao existe").to_vec(),
        );
    }

    pub fn run_main(&mut self) {
        self.run_function("main");
    }
}

// ---------- Main ----------

fn main() {
    log_error!("oi: {}", 5);
    let args: Vec<String> = env::args().collect();
    let mut estado = PSFState::new();

    if args.len() == 1 {
        // repl
        let mut input = String::new();
        loop {
            print!("> ");
            let _ = io::stdout().flush();
            io::stdin().read_line(&mut input).unwrap();
            input = input.trim().to_owned();

            match input.as_str() {
                ":e" | ":exit" => {
                    return;
                }
                outro => {
                    estado.run_raw_string(outro);
                    // println!("{:?}", estado.stack);
                    estado.clear_stack();
                }
            }
            input.clear();
        }
    } else if args.len() == 2 {
        // ler arquivo
        let conteudo = match fs::read_to_string(&args[1]) {
            Err(erro) => {
                println!("Erro abrindo arquivo: {}", erro);
                return;
            }
            Ok(str) => str,
        };
        estado.load_string(&conteudo);
        estado.run_main();
    } else {
        println!("3 nao existe");
        return;
    }
}
