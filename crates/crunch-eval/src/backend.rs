use std::ffi::OsString;

use crate::Context;
use crate::Error;
use crate::Expr;

pub(crate) struct EvalRequest<'a> {
    pub(crate) source: &'a str,
    pub(crate) import_paths: Vec<OsString>,
    pub(crate) source_name: String,
}

pub(crate) trait EvalBackend {
    fn eval(&self, request: EvalRequest<'_>) -> Result<Expr, Error>;

    fn eval_to_json(&self, request: EvalRequest<'_>) -> Result<String, Error>;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NickelBackend;

pub(crate) fn default_backend() -> NickelBackend {
    NickelBackend
}

impl EvalBackend for NickelBackend {
    fn eval(&self, request: EvalRequest<'_>) -> Result<Expr, Error> {
        let (_ctx, expr) = eval_with_context(request)?;
        Ok(expr)
    }

    fn eval_to_json(&self, request: EvalRequest<'_>) -> Result<String, Error> {
        let (ctx, expr) = eval_with_context(request)?;
        let json = ctx.expr_to_json(&expr)?;
        Ok(json)
    }
}

fn eval_with_context(request: EvalRequest<'_>) -> Result<(Context, Expr), Error> {
    let source_name = request.source_name;
    assert!(!source_name.is_empty(), "source name must not be empty");

    let mut ctx = Context::new().with_added_import_paths(request.import_paths).with_source_name(source_name);

    let expr = ctx.eval_deep_for_export(request.source)?;
    Ok((ctx, expr))
}
