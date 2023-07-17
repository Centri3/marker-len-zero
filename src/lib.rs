#![feature(let_chains)]
#![warn(clippy::pedantic)]

use marker_api::ast::expr::BinaryOpKind;
use marker_api::diagnostic::Applicability;
use marker_api::prelude::*;
use marker_api::{LintPass, LintPassInfo, LintPassInfoBuilder};

#[derive(Default)]
struct MyLintPass {}
marker_api::export_lint_pass!(MyLintPass);

marker_api::declare_lint! {
    /// # What it does
    /// Reimplementation of
    /// [`clippy::len_zero`](https://rust-lang.github.io/rust-clippy/master/index.html#/len_zero).
    ///
    /// # Example
    /// ```rs
    /// if x.len() == 0 {
    ///     ..
    /// }
    /// if y.len() != 0 {
    ///     ..
    /// }
    /// ```
    ///
    /// Use instead:
    /// ```rs
    /// if x.is_empty() {
    ///     ..
    /// }
    /// if !y.is_empty() {
    ///     ..
    /// }
    /// ```
    LEN_ZERO,
    Warn,
}

impl LintPass for MyLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([LEN_ZERO])).build()
    }

    fn check_expr<'ast>(&mut self, cx: &'ast AstContext<'ast>, expr: ExprKind<'ast>) {
        // TODO: Make sure we aren't in `is_empty`.
        // TODO: Make sure the type defines `is_empty`.

        if let ExprKind::BinaryOp(binop) = expr
            && matches!(binop.kind(), BinaryOpKind::Eq | BinaryOpKind::NotEq)
            && let ExprKind::Method(method) = binop.left()
            && method.method().ident().name() == "len"
            && let ExprKind::IntLit(compare_to) = binop.right()
            && (compare_to.value() == 0 || compare_to.value() == 1)
        {
            let op = match binop.kind() {
                BinaryOpKind::Eq => "",
                BinaryOpKind::NotEq => "!",
                _ => unreachable!(),
            };

            cx.emit_lint(
                LEN_ZERO,
                expr.id(),
                format!(
                    "length comparison to {}",
                    if compare_to.value() == 0 {
                        "zero"
                    } else {
                        "one"
                    }
                ),
                expr.span(),
                |diag| {
                    diag.span_suggestion(
                        format!("using `{op}is_empty` is clearer and more explicit"),
                        expr.span(),
                        format!("{op}{}.is_empty()", method.receiver().span().snippet_or("whoops")),
                        Applicability::Unspecified,
                    );
                },
            );
        }
    }
}
