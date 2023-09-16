use super::*;
use crate::error::{Error, ProjectError, RunnerError};
use crate::project::Container;
use crate::project::Script;
use crate::types::{ResourceId, ResourcePath};
use dev_utils::fs::temp_file;
use dev_utils::{create_lock, lock::get_lock};
use fake::faker::lorem::raw::Word;
use fake::locales::EN;
use fake::Fake;
use mockall::*;
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::{fs, str};

// ********************
// *** Runner Hooks ***
// ********************

#[test]
fn runner_hooks_new_should_work() {
    fn get_script(rid: &ResourceId) -> Result<Script> {
        Err(Error::ProjectError(ProjectError::NotRegistered(
            Some(rid.clone()),
            None,
        )))
    }

    let _hooks = RunnerHooks::new(get_script);
}

// **************
// *** Runner ***
// **************

#[test]
fn runner_new_should_work() {
    Runner::new(create_default_runner_hooks());
}

// ------------------
// --- run_script ---
// ------------------

#[test]
fn runner_run_script_should_work() {
    // setup
    let script = create_script("py");
    let key: String = Word(EN).fake();
    fs::write(&script.path, format!("print('{}')", key)).expect("could not write to file");
    let runner = Runner::new(create_default_runner_hooks());
    let container = Container::new(Word(EN).fake::<String>());

    // test
    let out = runner
        .run_script(script, &container)
        .expect("`run_script` should work");

    let stdout = str::from_utf8(out.stdout.as_slice()).expect("stdout should work");
    assert_eq!(stdout, format!("{}\n", key), "incorrect output");
}

#[test]
fn runner_run_script_if_script_errors_should_err() {
    // setup
    let script = create_script("py");
    let prg = r#"
        import sys
        sys.exit(1)
        "#;

    fs::write(&script.path, prg).expect("could not write to file");

    let runner = Runner::new(create_default_runner_hooks());
    let container = Container::new(Word(EN).fake::<String>());

    let sid = script.rid.clone();
    let cid = container.rid.clone();

    // test
    let res = runner.run_script(script, &container);

    assert!(res.is_err(), "runner did not error");
    let Err(Error::RunnerError(RunnerError::ScriptError(e_sid, e_cid, _msg))) = res else {
        panic!("incorrect error type");
    };

    assert_eq!(e_sid, sid, "incorrect script id");
    assert_eq!(e_cid, cid, "incorrect container id");
}

// -------------------
// --- run_scripts ---
// -------------------

#[test]
fn runner_run_scripts_should_work() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let key: String = Word(EN).fake();
    let prg = format!("print('{}')", key);
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.script_error = Some(TestHooks::script_error_err);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);

    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let script_error_ctx = TestHooks::script_error_err_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(num_scripts);
    script_error_ctx.expect().times(0);
    post_script_ctx.expect().times(num_scripts);

    runner
        .run_scripts(scripts, &container, false, false)
        .expect("`run_scripts` should work");
}

#[test]
#[should_panic(expected = "RunnerError")]
fn runner_run_scripts_with_unhandled_error_should_halt() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let prg = format!("raise RuntimeError()");
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);

    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(num_scripts);
    post_script_ctx.expect().never();

    let res = runner.run_scripts(scripts, &container, false, false);
    res.unwrap();
}

#[test]
fn runner_run_scripts_with_handled_error_that_returns_ok_should_work() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let prg = format!("raise RuntimeError()");
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.script_error = Some(TestHooks::script_error_ok);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);

    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let script_error_ctx = TestHooks::script_error_ok_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(num_scripts);
    script_error_ctx
        .expect()
        .times(num_scripts)
        .return_once(move |_ctx, _err, _verbose| Ok(()));

    post_script_ctx.expect().times(num_scripts);

    runner
        .run_scripts(scripts, &container, false, false)
        .expect("`run_scripts` should work");
}

#[test]
#[should_panic(expected = "RunnerError")]
fn runner_run_scripts_with_handled_error_that_returns_err_should_halt() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let prg = format!("raise RuntimeError()");
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.script_error = Some(TestHooks::script_error_err);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);

    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let script_error_ctx = TestHooks::script_error_err_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(scripts.len());
    script_error_ctx
        .expect()
        .times(num_scripts)
        .return_once(move |_ctx, err, _verbose| Err(err));

    post_script_ctx.expect().times(0);

    let res = runner.run_scripts(scripts, &container, false, false);
    res.unwrap();
}

#[test]
fn runner_run_scripts_with_unhandled_error_ignored_should_work() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let prg = format!("raise RuntimeError()");
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);

    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(num_scripts);
    post_script_ctx.expect().times(num_scripts);

    let _res = runner
        .run_scripts(scripts, &container, true, false)
        .expect("`run_scripts` should work");
}

#[test]
fn runner_run_scripts_with_handled_error_that_returns_err_ignored_should_work() {
    // setup
    let _m = get_lock(&MTX);

    let script = create_script("py");
    let prg = format!("raise RuntimeError()");
    fs::write(&script.path, prg).expect("could not write program to file");

    let container = Container::new(Word(EN).fake::<String>());
    let scripts = vec![script];
    let num_scripts = scripts.len();

    let mut hooks = create_default_runner_hooks();
    hooks.pre_script = Some(TestHooks::pre_script);
    hooks.assets_added = Some(TestHooks::assets_added);
    hooks.script_error = Some(TestHooks::script_error_err);
    hooks.post_script = Some(TestHooks::post_script);

    let runner = Runner::new(hooks);
    // test
    let pre_script_ctx = TestHooks::pre_script_context();
    let assets_added_ctx = TestHooks::assets_added_context();
    let script_error_ctx = TestHooks::script_error_err_context();
    let post_script_ctx = TestHooks::post_script_context();

    pre_script_ctx.expect().times(num_scripts);
    assets_added_ctx.expect().times(num_scripts);
    script_error_ctx
        .expect()
        .times(num_scripts)
        .return_once(move |_ctx, err, _verbose| Err(err));

    post_script_ctx.expect().times(num_scripts);

    runner
        .run_scripts(scripts, &container, true, false)
        .expect("`run_scripts` should work");
}

// ************************
// *** helper functions ***
// ************************

/// Creates default runner hooks.
fn create_default_runner_hooks() -> RunnerHooks {
    fn get_script(rid: &ResourceId) -> Result<Script> {
        Err(Error::ProjectError(ProjectError::NotRegistered(
            Some(rid.clone()),
            None,
        )))
    }

    RunnerHooks::new(get_script)
}

/// Create a script to run.
fn create_script(ext: &str) -> Script {
    let path = temp_file::mkfile_with_extension(ext).expect("could not create script file");
    let path = ResourcePath::new(path).expect("could not parse path into `ResourcePath`");
    let script = Script::new(path).expect("could not create `Script`");
    script
}

// ******************
// *** Mock Setup ***
// ******************
// See https://github.com/asomers/mockall/blob/master/mockall/examples/synchronization.rs
// for details.

create_lock!(MTX);

#[mockall_double::double]
use test_hooks::TestHooks;

// -----------------
// --- TestHooks ---
// -----------------

mod test_hooks {
    use super::*;

    /// Default test hooks
    ///
    /// # Note
    /// `mockall` renames the struct to `TestHooks`,
    /// which is what should be used for testing.
    #[derive(Default)]
    pub(super) struct TestHooks {}

    #[automock]
    impl TestHooks {
        pub fn pre_script(ctx: ScriptExecutionContext, verbose: bool) {
            if verbose {
                dbg!("pre: {:#?}", ctx);
            }
        }

        /// Error handler that always returns `Ok`.
        pub fn script_error_ok(
            ctx: ScriptExecutionContext,
            err: RunnerError,
            verbose: bool,
        ) -> StdResult<(), RunnerError> {
            if verbose {
                dbg!("error: {:#?} {:#?}", ctx, err);
            }

            Ok(())
        }

        /// Error handler that always returns `Err` with the provided [`RunnerError`].
        pub fn script_error_err(
            ctx: ScriptExecutionContext,
            err: RunnerError,
            verbose: bool,
        ) -> StdResult<(), RunnerError> {
            if verbose {
                dbg!("error: {:#?} {:#?}", ctx, &err);
            }

            Err(err)
        }

        pub fn assets_added(
            ctx: ScriptExecutionContext,
            assets: HashSet<ResourceId>,
            verbose: bool,
        ) {
            if verbose {
                dbg!("assets: {:#?} {:#?}", ctx, assets);
            }
        }

        pub fn post_script(ctx: ScriptExecutionContext, verbose: bool) {
            if verbose {
                dbg!("post: {:#?}", ctx);
            }
        }
    }
}
