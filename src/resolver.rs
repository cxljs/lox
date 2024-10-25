use std::collections::HashMap;

use crate::{
    ast::{Expr, Stmt},
    error::Error,
    token::Token,
};

pub fn resolve_variable(stmts: &Vec<Stmt>) -> Option<HashMap<Expr, usize>> {
    let mut resolver = Resolver::new();
    resolver.resolve_stmts(stmts);
    if resolver.has_err {
        return None;
    }
    Some(resolver.depths)
}

#[derive(Clone, Copy, PartialEq)]
enum FuncType {
    NONE,
    FUNCTION,
}

// 执行 ast 时，对函数闭包的捕获有一个 bug: 捕获的闭包后续的修改可能会导致函数读取到错误的变量/函数.
// 比如一下这段代码，Lox (包括其他语言)期望的行为应该是打印2次 `global`,
// 但是 commit(ac940b4) 的行为是打印 `global`和`block`.
// var a = "global";
// {
//   fun showA() {
//     print a;
//   }

//   showA();
//   var a = "block";
//   showA();
// }
//
// 我们目前所有的 env 形成一条链，如果我们用一个可持久化数据结构来实现 env，能够很好的处理上面的问题.
// 但是这里采用一个更加容易实现的方案: 在执行 ast 前，对 ast 做一次语法分析，对于每一个变量、函数，记录
// 它绑定的值在 env 链的哪一个节点，读取变量、函数的值时直接到目标节点去读取.
// Resolver 关心4种场景:
// 1. block 语句会进入新的 env (env 链建立一个新的节点)
// 2. 执行函数也会进入新的 env
// 3. 变量声明会在当前 env 加入新值
// 4. 读取变量的表达式会解析变量的值.
struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    curr_func: FuncType,
    depths: HashMap<Expr, usize>,
    has_err: bool,
}

impl Resolver {
    fn new() -> Self {
        Self {
            scopes: Vec::new(),
            curr_func: FuncType::NONE,
            depths: HashMap::new(),
            has_err: false,
        }
    }

    fn resolve_stmts(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { stmts } => {
                self.begin_scope();
                self.resolve_stmts(stmts);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init);
                }
                self.define(name);
            }
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.resolve_func(params, body, FuncType::FUNCTION);
            }
            Stmt::Expression { expr } => {
                self.resolve_expr(expr);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            }
            Stmt::Print { expr } => self.resolve_expr(expr),
            Stmt::Return { keyword, value } => {
                // return 语句在函数内部才有意义.
                if self.curr_func == FuncType::NONE {
                    eprintln!(
                        "{}",
                        Error::ParseError(
                            keyword.clone(),
                            "Can't return from top-level code.".to_string(),
                        )
                    );
                    self.has_err = true;
                }
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
            Stmt::Class { .. } => todo!(),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty()
                    && !self
                        .scopes
                        .last()
                        .unwrap()
                        .get(&name.lexeme)
                        .unwrap_or(&true)
                {
                    eprintln!(
                        "{}",
                        Error::ParseError(
                            name.clone(),
                            "Can't read local variable in its own initializer.".to_string(),
                        )
                    );
                    self.has_err = true;
                }
                self.resolve_local(expr, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
            }
            Expr::Binary { left, op: _, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Grouping { expression: _ } => self.resolve_expr(expr),
            Expr::Literal { value: _ } => (),
            Expr::Logical { left, op: _, right } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Unary { op: _, right } => self.resolve_expr(right),
            _ => todo!(),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            let cur = &self.scopes[i];
            if cur.contains_key(&name.lexeme) {
                self.depths.insert(expr.clone(), self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_func(&mut self, params: &Vec<Token>, body: &Vec<Stmt>, func_type: FuncType) {
        let enclosing_func = self.curr_func;
        self.curr_func = func_type;
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(body);
        self.end_scope();
        self.curr_func = enclosing_func;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let cur = self.scopes.last_mut().unwrap();

        // Lox 不允许在 local scope 重复定义变量 (和大部分语言一样)，
        // 但是 Lox 允许在 global scope 重复定义变量，这点设计不太赞同.
        if cur.contains_key(&name.lexeme) {
            eprintln!(
                "{}",
                Error::ParseError(
                    name.clone(),
                    "Already a variable with this name is this scope.".to_string(),
                )
            );
            self.has_err = true;
        }

        cur.insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), true);
    }
}
