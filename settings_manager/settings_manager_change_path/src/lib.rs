/**
 * Attribute macro to change the path of SettingsManager's.
 */
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::path::PathBuf;
use syn::fold::Fold;
use syn::token::Brace;
use syn::{
    parse2, parse_macro_input, AttributeArgs, Block, Expr, ExprBlock, ExprCall, ExprMethodCall,
    ItemFn, Lit, NestedMeta, Stmt, Token,
};
use tempfile::NamedTempFile;

struct ReplacePathCalls {
    var_name: String,
    path: PathBuf,
    method: String,
}

impl ReplacePathCalls {
    pub fn new(var_name: String, path: PathBuf) -> ReplacePathCalls {
        ReplacePathCalls {
            var_name,
            path,
            method: String::from("path"),
        }
    }

    fn should_replace_path(&self, expr: &ExprMethodCall) -> bool {
        // match receiver
        let valid_recv = match &*expr.receiver {
            Expr::Path(e_path) => match e_path.path.segments.first() {
                Some(seg) => seg.ident == self.var_name,
                None => false,
            },

            _ => return false,
        };

        // match method, #path
        let valid_method = expr.method == self.method;

        valid_recv && valid_method
    }

    fn replace_path(&self, expr: ExprMethodCall) -> Block {
        let semi_token = Token![;];
        let o_stmt = Stmt::Semi(Expr::MethodCall(expr), semi_token(Span::call_site()));
        let path_stmt = Stmt::Expr(Expr::Call(self.path_expr()));

        let brace_token = Brace {
            span: Span::call_site(),
        };

        let mut stmts = Vec::<Stmt>::new();
        stmts.push(o_stmt);
        stmts.push(path_stmt);

        Block { brace_token, stmts }
    }

    fn path_expr(&self) -> ExprCall {
        let l_path = match self.path.to_str() {
            Some(p) => p,
            None => panic!("Invalid path"),
        };

        let expr = quote! {Ok(PathBuf::from(#l_path))};
        let expr = match parse2(expr) {
            Err(err) => panic!("{:?} Could not parse new PathBuf.", err),
            Ok(res) => res,
        };

        expr
    }
}

impl Fold for ReplacePathCalls {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::MethodCall(ref e_call) => {
                if self.should_replace_path(e_call) {
                    Expr::Block(ExprBlock {
                        attrs: Vec::new(),
                        label: None,
                        block: self.replace_path(e_call.clone()),
                    })
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }
}

/**
 * Change the path of a SettingsManager.
 *
 * # Parameters
 * 1. **SettingsManager Instance:** Variable of the SettingsManager to target.
 * 2. **Replacement Path:** (optional) Path to replace with, otherwise is random.
 */
#[proc_macro_attribute]
pub fn settings_path(args: TokenStream, input: TokenStream) -> TokenStream {
    // parse args
    let p_args = parse_macro_input!(args as AttributeArgs);
    if p_args.is_empty() {
        // compile_error!("Must pass variable name");
        panic!("Must provide variable name");
    }

    let mut i_args = p_args.into_iter();

    // first arg is vairable name
    let v_name = match i_args.next() {
        Some(NestedMeta::Meta(v)) => {
            // @todo [1]: Allow for Meta to be passed
            //      e.g. #settings_path(a)
            //      i.e. without quotes.
            panic!("Invalid varibale name {:?}", v)
        }

        Some(NestedMeta::Lit(v)) => match v {
            Lit::Str(vn) => vn,
            _ => panic!("Invalid variable name {:?}", v),
        },

        v => panic!("Invalid variable name {:?}", v),
    };

    // second arg is desired path if given
    // otherwise create temporary file
    let path = match i_args.next() {
        None => {
            // file not give, create path from temp file
            match NamedTempFile::new() {
                Ok(f) => f.path().to_path_buf(),
                Err(err) => {
                    panic!("Could not create temporary file: {:?}", err);
                }
            }
        }
        Some(NestedMeta::Lit(p)) => match p {
            Lit::Str(ps) => PathBuf::from(ps.value()),
            _ => panic!("Invalid path {:?}", p),
        },
        p => panic!("Invalid path {:?}", p),
    };

    // parse function
    let mut path_replacer = ReplacePathCalls::new(v_name.value(), path.clone());
    let ast = parse_macro_input!(input as ItemFn);

    let ast = path_replacer.fold_item_fn(ast);
    ast.into_token_stream().into()
}
