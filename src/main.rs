use std::collections::HashMap;
use std::io::Write;
use std::{env, fs, io, str};

// ---------- TODO ----------

// - tokens de mais de 1 simbolo como == <= >= !=
// - criacao de listas com []
// - pensar em criacao de structs

// ---------- Logging ----------

#[allow(unused)]
macro_rules! log_info {
    ( $($arg:tt)* ) => {
        print!("\x1b[1;34minfo\x1b[0m: ");
        println!($($arg)*);
    };
}

macro_rules! log_error {
    // () => {
    //     println!("\x1b[1;31merro\x1b[0m");
    //     std::process::exit(0);
    // };
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
    Menor,
    Maior,
    // numeros e nomes
    // keywords sao reconhecidas na criacao da ast mas queria ter feito aq
    Numero(i32),
    Simbolo(String),
    String(String),
}

fn tokenizar(entrada: &str) -> Vec<Token> {
    let mut tokens = vec![];

    let mut buffer = String::new();
    #[derive(PartialEq)]
    enum OQUE {
        Nada,
        Numero,
        Simbolo,
        String,
    }
    let mut oqe = OQUE::Nada;

    for c in entrada.chars() {
        // log_info!("char: |{}|", c);
        if oqe == OQUE::Numero {
            match c {
                c if c.is_digit(10) => {
                    buffer.push(c);
                    continue;
                }
                c if c.is_alphabetic() => {
                    // log_error!("nao pode letra dps de numero: {:?}", c);
                }
                _ => {
                    tokens.push(Token::Numero(match buffer.parse() {
                        Ok(n) => n,
                        Err(err) => log_error!("erro no parse de numero {}", err),
                    }));
                    // log_info!("terminou numero: |{}|", buffer);
                    buffer.clear();
                    oqe = OQUE::Nada;
                }
            }
        } else if oqe == OQUE::Simbolo {
            match c {
                c if (c.is_alphabetic() | c.is_digit(10)) => {
                    buffer.push(c);
                    continue;
                }
                _ => {
                    tokens.push(Token::Simbolo(String::from(&buffer)));
                    // log_info!("terminou simbolo: |{}|", buffer);
                    buffer.clear();
                    oqe = OQUE::Nada;
                }
            }
        } else if oqe == OQUE::String {
            if c == '"' {
                tokens.push(Token::String(String::from(&buffer)));
                // log_info!("terminou string: |{}|", buffer);
                buffer.clear();
                oqe = OQUE::Nada;
            } else {
                buffer.push(c);
            }
            continue;
        }
        match c {
            '=' => tokens.push(Token::Igual),
            '(' => tokens.push(Token::ParenAbr),
            ')' => tokens.push(Token::ParenFec),
            '?' => tokens.push(Token::Interrogacao),
            '!' => tokens.push(Token::Exclamacao),
            '+' => tokens.push(Token::Mais),
            '>' => tokens.push(Token::Maior),
            '<' => tokens.push(Token::Menor),
            '"' => {
                // log_info!("comecou string");
                oqe = OQUE::String;
            }
            c if c.is_digit(10) => {
                // log_info!("comecou numero");
                buffer.push(c);
                oqe = OQUE::Numero;
            }
            c if c.is_alphabetic() => {
                // log_info!("comecou simbolo");
                buffer.push(c);
                oqe = OQUE::Simbolo;
            }
            c if c.is_whitespace() => (),
            outro => {
                log_error!("nao sei: {:?}", outro);
            }
        }
    }

    if !buffer.is_empty() {
        if oqe == OQUE::Numero {
            tokens.push(Token::Numero(match buffer.parse() {
                Ok(n) => n,
                Err(err) => log_error!("nao deveria dar errado: {}", err),
            }));
        } else if oqe == OQUE::Simbolo {
            tokens.push(Token::Simbolo(String::from(&buffer)));
        }
    }
    // log_info!("tokens: {:?}", tokens);
    tokens
}

// ---------- AST ----------

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
    Maior,
    Menor,
    // funcoes builin
    Print,
    Input,
    Pop,
    Dup,
    Swap,
    SwapN,
    SSize,
    If,
    DebugS,
    // literais
    True,
    False,
    Numero(i32),
    // funcoes
    String(String),
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
            Token::Maior => {
                funcao_atual.push(ASTItem::Maior);
            }
            Token::Menor => {
                funcao_atual.push(ASTItem::Menor);
            }
            Token::Numero(n) => {
                funcao_atual.push(ASTItem::Numero(*n));
            }
            Token::Simbolo(nome) => match nome.as_str() {
                "print" => funcao_atual.push(ASTItem::Print),
                "input" => funcao_atual.push(ASTItem::Input),
                "pop" => funcao_atual.push(ASTItem::Pop),
                "dup" => funcao_atual.push(ASTItem::Dup),
                "swap" => funcao_atual.push(ASTItem::Swap),
                "swapn" => funcao_atual.push(ASTItem::SwapN),
                "ssize" => funcao_atual.push(ASTItem::SSize),
                "if" => funcao_atual.push(ASTItem::If),
                "debugs" => funcao_atual.push(ASTItem::DebugS),
                "true" => funcao_atual.push(ASTItem::True),
                "false" => funcao_atual.push(ASTItem::False),
                _ => funcao_atual.push(ASTItem::FuncCallNamed(nome.to_string())),
            },
            Token::String(s) => {
                funcao_atual.push(ASTItem::String(s.clone()));
            }
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

// ---------- Interpretacao ----------

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Item {
    Bool(bool),
    Numero(i32),
    String(String),
    Func(Vec<ASTItem>),
}

pub fn interpretar_func(estado: &mut PSFState, func: Func) {
    let mut stack_consumir = Stack::new();
    for item in func.iter().rev() {
        stack_consumir.push(item.clone());
    }
    let mut item: ASTItem;
    loop {
        // log_info!(
        //     "\nstack: {:?}\ncons: {:?}",
        //     estado.stack.lista,
        //     stack_consumir.lista
        // );
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
            ASTItem::String(s) => {
                estado.stack.push(Item::String(s));
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
            ASTItem::Maior => {
                let Some(Item::Numero(n1)) = estado.stack.pop() else {
                    log_error!("tipo do primeiro argumento nao e numero");
                };
                let Some(Item::Numero(n2)) = estado.stack.pop() else {
                    log_error!("tipo do segundo argumento nao e numero");
                };
                estado.stack.push(Item::Numero(n2));
                estado.stack.push(Item::Numero(n1));
                estado.stack.push(Item::Bool(n2 > n1));
            }
            ASTItem::Menor => {
                let Some(Item::Numero(n1)) = estado.stack.pop() else {
                    log_error!("tipo do primeiro argumento nao e numero");
                };
                let Some(Item::Numero(n2)) = estado.stack.pop() else {
                    log_error!("tipo do segundo argumento nao e numero");
                };
                estado.stack.push(Item::Numero(n2));
                estado.stack.push(Item::Numero(n1));
                estado.stack.push(Item::Bool(n2 < n1));
            }
            ASTItem::Print => {
                let Some(item) = estado.stack.pop() else {
                    log_error!("stack vazia antes do print");
                };
                estado.stack.push(item.clone());
                // log_info!("item do print: {:?}", item);
                match item {
                    Item::Numero(n) => println!("{}", n),
                    Item::Bool(b) => println!("{}", b),
                    Item::String(s) => println!("{}", s),
                    outro => {
                        println!("impossivel: {:?}", outro);
                    }
                }
            }
            ASTItem::Input => {
                let Some(item) = estado.stack.pop() else {
                    log_error!("stack vaiz antes do input");
                };
                let Item::String(s) = item else {
                    log_error!("input so aceita string como entrada");
                };
                print!("{}", s);
                let mut input = String::new();
                _ = io::stdout().flush();
                io::stdin().read_line(&mut input).unwrap();
                estado.stack.push(Item::String(input));
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
            ASTItem::SwapN => {
                let Some(posi) = estado.stack.pop() else {
                    log_error!("stack vazia na chamada do swapn");
                };
                let Item::Numero(pos) = posi else {
                    log_error!("topo da stack nao e numero");
                };
                if pos < 0 {
                    log_error!("impossivel trocar posicao negativa: {}", pos);
                }
                let tam = estado.stack.len() as i32;
                if pos >= tam {
                    log_error!(
                        "tentando indexar {} mas a stack ta com tamanho {}",
                        pos,
                        tam
                    );
                } else if pos == 0 {
                    return;
                }
                let topi = estado.stack.lista[tam as usize - 1].clone();
                let nth = estado.stack.lista[tam as usize - 1 - pos as usize].clone();
                estado.stack.lista[tam as usize - 1] = nth;
                estado.stack.lista[tam as usize - 1 - pos as usize] = topi;
            }
            ASTItem::SSize => {
                estado.stack.push(Item::Numero(estado.stack.len() as i32));
            }
            ASTItem::If => {
                let Some(iff) = estado.stack.pop() else {
                    log_error!("stack vazia na chamada do if");
                };
                let Some(ifv) = estado.stack.pop() else {
                    log_error!("stack com somente um elemento no if");
                };
                let Some(ibo) = estado.stack.pop() else {
                    log_error!("stack sem terceiro item no if");
                };
                let Item::Func(ff) = iff else {
                    log_error!("funcao do else nao e funcao");
                };
                let Item::Func(fv) = ifv else {
                    log_error!("funcao do if nao e funcao");
                };
                let Item::Bool(bo) = ibo else {
                    log_error!("valor do if nao e booleano");
                };
                if bo {
                    estado.stack.push(Item::Func(fv));
                } else {
                    estado.stack.push(Item::Func(ff));
                }
                stack_consumir.push(ASTItem::FuncCallTop);
            }
            ASTItem::DebugS => {
                println!("debug: {:?}", estado.stack);
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

    pub fn load_funcs(&mut self) {
        self.funcoes
            .insert("printp".to_owned(), vec![ASTItem::Print, ASTItem::Pop]);
    }
}

// ---------- Helpers ----------

fn tokenizar_e_gerar_ast(entrada: &str, funcao: bool) -> AST {
    gerar_ast(tokenizar(entrada), funcao)
}

fn print_usage() {
    println!(
        "[ARQUIVO | [OPCOES]*]
OPCOES:
    -h help
    -i interativo"
    );
}

fn print_usage_repl() {
    println!(
        "usagem
    :h :help :?   print help
    :e :exit      exit"
    );
}

fn run(args: Vec<String>, estado: &mut PSFState) {
    let mut tem_arq: Option<String> = None;
    let mut repl = false;
    let mut i = 1;

    if args.len() == 1 {
        // print_usage();
        // return;
        repl = true;
    }

    while i < args.len() {
        let arg = args[i].clone();
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "-i" => {
                repl = true;
            }
            outro => {
                if let Ok(tem) = fs::exists(outro) {
                    if tem {
                        tem_arq = Some(arg.clone());
                    }
                } else {
                    log_error!("arg nao reconhecido: {}", outro);
                }
            }
        }
        i += 1;
    }
    if let Some(arq) = tem_arq {
        // ler arquivo
        let conteudo = match fs::read_to_string(arq) {
            Err(erro) => {
                println!("Erro abrindo arquivo: {}", erro);
                return;
            }
            Ok(str) => str,
        };
        estado.load_string(&conteudo);
        estado.run_main();
    }

    if repl {
        // repl
        let mut input = String::new();
        loop {
            print!("> ");
            let _ = io::stdout().flush();
            io::stdin().read_line(&mut input).unwrap();
            input = input.trim().to_owned();

            match input.as_str() {
                ":h" | ":help" | ":?" => {
                    print_usage_repl();
                }
                ":e" | ":exit" => {
                    return;
                }
                outro => {
                    estado.run_raw_string(outro);
                    // log_info!("stack: {:?}", estado.stack.lista);
                    estado.clear_stack();
                }
            }
            input.clear();
        }
    }
}

// ---------- Main ----------

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut estado = PSFState::new();
    estado.load_funcs();

    run(args, &mut estado);
}
