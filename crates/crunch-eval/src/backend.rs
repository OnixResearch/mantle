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

#[cfg(feature = "cranelift-proto")]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CraneliftPrototypeBackend;

pub(crate) fn default_backend() -> NickelBackend {
    NickelBackend
}

#[cfg(feature = "cranelift-proto")]
pub(crate) fn cranelift_prototype_backend() -> CraneliftPrototypeBackend {
    CraneliftPrototypeBackend
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

#[cfg(feature = "cranelift-proto")]
impl EvalBackend for CraneliftPrototypeBackend {
    fn eval(&self, _request: EvalRequest<'_>) -> Result<Expr, Error> {
        Err(Error::Serde(
            "cranelift prototype backend only supports JSON export for flat derivation literals".to_string(),
        ))
    }

    fn eval_to_json(&self, request: EvalRequest<'_>) -> Result<String, Error> {
        if !request.import_paths.is_empty() {
            return Err(Error::Serde("cranelift prototype backend does not support import paths".to_string()));
        }
        crate::cranelift_proto::evaluate_flat_derivation_to_json(request.source)
            .map_err(|err| Error::Serde(err.to_string()))
    }
}

fn eval_with_context(request: EvalRequest<'_>) -> Result<(Context, Expr), Error> {
    let source_name = request.source_name;
    assert!(!source_name.is_empty(), "source name must not be empty");

    let mut ctx = Context::new().with_added_import_paths(request.import_paths).with_source_name(source_name);

    let expr = ctx.eval_deep_for_export(request.source)?;
    Ok((ctx, expr))
}
