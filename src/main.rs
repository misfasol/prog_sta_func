use std::collections::HashMap;
use std::io::Write;
use std::{env, fs, io, str};

// ---------- Logging ----------

#[allow(unused)]
macro_rules! log_info {
    ( $($arg:tt)* ) => {
        print!("\x1b[1;34minfo\x1b[0m: ");
        println!($($arg)*);
    };
}

macro_rules! log_error {
    () => {
        println!("\x1b[1;31merro\x1b[0m");
    };
    ( $($arg:tt)* ) => {{
        print!("\x1b[1;31merro\x1b[0m: ");
        println!($($arg)*);
        std::process::exit(0);
    }};
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
    Mais,
    // Menor,
    // Maior,
    // Comp,
    // numeros e nomes
    // keywords sao reconhecidas na criacao da ast mas queria ter feito aq
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
                    c if c.is_digit(10) => {
                        buffer.push(c);
                    }
                    c if c.is_alphabetic() => {
                        log_error!("nao pode letra dps de numero: {:?}", c);
                    }
                    _ => {
                        tokens.push(Token::Numero(match buffer.parse() {
                            Ok(n) => n,
                            Err(err) => log_error!("{}", err),
                        }));
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
                        // match &buffer[..] {
                        //     "true" => tokens.push(Token::True),
                        //     "false" => tokens.push(Token::False),
                        //     _ => {
                        //         log_info!("ta: {}", buffer);
                        //         tokens.push(Token::Simbolo(String::from(&buffer)));
                        //     }
                        // }
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
                    log_error!("nao sei: {:?}", outro);
                }
            }
        }
    }

    if !buffer.is_empty() {
        if enumero {
            tokens.push(Token::Numero(match buffer.parse() {
                Ok(n) => n,
                Err(err) => log_error!("nao deveria dar errado: {}", err),
            }));
        } else {
            tokens.push(Token::Simbolo(String::from(&buffer)));
        }
    }
    // log_info!("tokens: {:?}", tokens);
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
    // operadores
    Mais,
    // builins
    Print,
    Pop,
    Dup,
    Swap,
    // literais
    True,
    False,
    // funcoes
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
                    log_error!("parenteses sem ter interrogacao antes");
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
                            log_error!("erro no parenfec");
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
            // Token::True => {
            //     funcao_atual.push(ASTItem::True);
            // }
            // Token::False => {
            //     funcao_atual.push(ASTItem::False);
            // }
            Token::Numero(n) => {
                funcao_atual.push(ASTItem::Numero(*n));
            }
            Token::Simbolo(nome) => match &nome[..] {
                "print" => funcao_atual.push(ASTItem::Print),
                "pop" => funcao_atual.push(ASTItem::Pop),
                "dup" => funcao_atual.push(ASTItem::Dup),
                "swap" => funcao_atual.push(ASTItem::Swap),
                "true" => funcao_atual.push(ASTItem::True),
                "false" => funcao_atual.push(ASTItem::False),
                _ => funcao_atual.push(ASTItem::FuncCallNamed(nome.to_string())),
            },
            Token::Igual => {
                log_error!("impossível ter igual dentro de uma funcao");
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
                log_error!("falta nome no começo de uma funcao");
            };
            i += 1;
            let Token::Igual = &tokens[i] else {
                log_error!("falta um igual na definicao da funcao {}", nome);
            };
            i += 1;
            let Token::ParenAbr = &tokens[i] else {
                log_error!("falta um parenteses no comeco da funcao {}", nome);
            };
            i += 1;
            let func = gerar_ast_funcao(&tokens, &mut i);
            let Token::ParenFec = &tokens[i] else {
                log_error!("falta um parenteses no final da funcao {}", nome);
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
    Bool(bool),
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
            log_error!("stack muito grande, terminando programa");
        }
        item = match stack_consumir.pop() {
            Some(i) => i,
            None => break,
        };
        match item {
            ASTItem::True => {
                estado.stack.push(Item::Bool(true));
            }
            ASTItem::False => {
                estado.stack.push(Item::Bool(false));
            }
            ASTItem::Numero(n) => {
                estado.stack.push(Item::Numero(n));
            }
            ASTItem::Mais => {
                let Some(Item::Numero(n1)) = estado.stack.pop() else {
                    log_error!("tipo do primeiro argumento nao e numero");
                };
                let Some(Item::Numero(n2)) = estado.stack.pop() else {
                    log_error!("tipo do segundo argumento nao e numero");
                };
                estado.stack.push(Item::Numero(n1 + n2));
            }
            ASTItem::Print => {
                let Some(item) = estado.stack.pop() else {
                    log_error!("stack vazia antes do print");
                };
                estado.stack.push(item.clone());
                match item {
                    Item::Numero(n) => println!("{}", n),

                    Item::Bool(b) => println!("{}", b),
                    outro => {
                        println!("impossivel: {:?}", outro);
                    }
                }
            }
            ASTItem::Pop => {
                _ = estado.stack.pop();
            }
            ASTItem::Dup => {
                let Some(topo) = estado.stack.pop() else {
                    log_error!("stack vazia na chamada do dup");
                };
                estado.stack.push(topo.clone());
                estado.stack.push(topo);
            }
            ASTItem::Swap => {
                let Some(prim) = estado.stack.pop() else {
                    log_error!("stack vazia na chamada do swap");
                };
                let Some(seg) = estado.stack.pop() else {
                    log_error!("stack com somente um elemento na chamada do swap");
                };
                estado.stack.push(prim);
                estado.stack.push(seg);
            }
            ASTItem::FuncDef(f) => {
                estado.stack.push(Item::Func(f));
            }
            ASTItem::FuncCallTop => {
                let Some(f) = estado.stack.pop() else {
                    log_error!("tentativa de chamar topo da funcao mas nao tem item");
                };
                let Item::Func(f) = f else {
                    log_error!("topo da funcao nao é funcao");
                };
                for i in f.iter().rev() {
                    stack_consumir.push(i.clone());
                }
            }
            ASTItem::FuncCallNamed(f) => {
                let Some(f) = estado.funcoes.get(&f) else {
                    log_error!("funcao `{:?}` nao existe", f);
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
                    log_info!("stack: {:?}", estado.stack.lista);
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
