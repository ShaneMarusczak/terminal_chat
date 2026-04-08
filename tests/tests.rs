#![allow(clippy::panic)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use tc::cli::{Mode, parse_cli_args};
use tc::commands::gf::load_files;
use tc::commands::handle_commands::{find_matching_word, min_distance, split_args};
use tc::message_printer::parse_color;
use tc::utils::{
    deduplicate_models, filter_models, get_base_model_name, get_default_model, has_date_suffix,
    sequence_equals, walk_directory,
};
use tc::*;

// ============================================================================
// Test Helpers
// ============================================================================

static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Self-cleaning temp directory. The directory and all its contents are
/// removed when this value is dropped.
struct TmpDir(PathBuf);

impl TmpDir {
    fn new(label: &str) -> Self {
        let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("tc-tests-{}-{}-{}", label, pid, n));
        fs::create_dir_all(&dir).expect("create temp dir");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn path_str(&self) -> &str {
        self.0.to_str().expect("temp dir path is utf-8")
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.0.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, content).expect("write file");
        path
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn s(v: &str) -> String {
    v.to_string()
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn path_string(path: &Path) -> String {
    path.to_str().expect("utf-8 path").to_string()
}

fn create_test_context() -> ConversationContext {
    let mut ctx = ConversationContext::new("gpt-4o");
    ctx.input.push(Message {
        role: Role::Developer,
        content: "You are a helpful assistant.".to_string(),
    });
    ctx.input.push(Message {
        role: Role::User,
        content: "Hello!".to_string(),
    });
    ctx.input.push(Message {
        role: Role::Assistant,
        content: "Hi! How can I help?".to_string(),
    });
    ctx
}

// ============================================================================
// Conversation / Provider / Request Tests
// ============================================================================

#[test]
fn test_provider_from_model_name() {
    assert_eq!(Provider::from_model_name("claude-3-5-sonnet"), Provider::Anthropic);
    assert_eq!(Provider::from_model_name("claude-opus-4"), Provider::Anthropic);
    assert_eq!(Provider::from_model_name("gpt-4o"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("o1"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("GPT-4"), Provider::OpenAI);
    assert_eq!(Provider::from_model_name("CLAUDE-3"), Provider::Anthropic);
}

#[test]
fn test_provider_local() {
    assert_eq!(Provider::from_model_name("local/qwen3.5-122b"), Provider::Local);
    assert_eq!(Provider::from_model_name("local/llama-3"), Provider::Local);
    assert_eq!(Provider::from_model_name("LOCAL/anything"), Provider::Local);
    assert_eq!(Provider::from_model_name("not-local/model"), Provider::OpenAI);
}

#[test]
fn test_openai_request_developer_mapping() {
    let ctx = create_test_context();
    let request = OpenAIRequest::new("gpt-4o", &ctx, None);

    assert_eq!(request.model, "gpt-4o");
    assert_eq!(request.messages.len(), 3);
    assert_eq!(request.messages[0].role, Role::System);
    assert_eq!(request.messages[0].content, "You are a helpful assistant.");
    assert_eq!(request.messages[1].role, Role::User);
    assert_eq!(request.messages[1].content, "Hello!");
    assert_eq!(request.messages[2].role, Role::Assistant);
}

#[test]
fn test_openai_request_model_override() {
    let ctx = create_test_context();
    let request = OpenAIRequest::new("local/qwen3.5", &ctx, None);
    assert_eq!(request.model, "local/qwen3.5");
}

#[test]
fn test_openai_request_max_tokens_serialization() {
    let ctx = create_test_context();

    let request = OpenAIRequest::new("gpt-4o", &ctx, Some(4096));
    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(json.contains("\"max_tokens\":4096"));

    let request = OpenAIRequest::new("gpt-4o", &ctx, None);
    let json = serde_json::to_string(&request).expect("Failed to serialize");
    assert!(!json.contains("max_tokens"));
}

#[test]
fn test_openai_request_serialization() {
    let ctx = create_test_context();
    let request = OpenAIRequest::new("gpt-4o", &ctx, None);

    let json = match serde_json::to_string(&request) {
        Ok(json) => json,
        Err(e) => panic!("Failed to serialize OpenAIRequest: {}", e),
    };
    assert!(json.contains("\"messages\""));
    assert!(!json.contains("\"input\""));
}

#[test]
fn test_anthropic_request_from_context() {
    let ctx = create_test_context();
    let request = AnthropicRequest::from_context(&ctx, 4096);

    assert_eq!(request.model, "gpt-4o");
    assert_eq!(request.max_tokens, 4096);
    assert_eq!(request.system, "You are a helpful assistant.");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, Role::User);
    assert_eq!(request.messages[1].role, Role::Assistant);
}

#[test]
fn test_anthropic_request_no_developer_message() {
    let mut ctx = ConversationContext::new("claude-3-5-sonnet");
    ctx.input.push(Message {
        role: Role::User,
        content: "Hi".to_string(),
    });
    let request = AnthropicRequest::from_context(&ctx, 1024);
    assert_eq!(request.system, "");
    assert_eq!(request.messages.len(), 1);
}

#[test]
fn test_conversation_context_new() {
    let ctx = ConversationContext::new("claude-3-5-sonnet");
    assert_eq!(ctx.model, "claude-3-5-sonnet");
    assert!(ctx.input.is_empty());
    assert!(ctx.yank_target.is_none());
}

#[test]
fn test_yank_target_persistence() {
    let mut ctx = ConversationContext::new("gpt-4o");
    assert!(ctx.yank_target.is_none());

    ctx.yank_target = Some("Test content".to_string());
    assert_eq!(ctx.yank_target, Some("Test content".to_string()));
}

#[test]
fn test_role_serialization() {
    let msg = Message {
        role: Role::Developer,
        content: "test".to_string(),
    };
    let json = serde_json::to_string(&msg).expect("Failed to serialize");
    assert!(json.contains("\"role\":\"developer\""));

    let msg = Message {
        role: Role::User,
        content: "test".to_string(),
    };
    let json = serde_json::to_string(&msg).expect("Failed to serialize");
    assert!(json.contains("\"role\":\"user\""));
}

#[test]
fn test_role_deserialization() {
    let json = r#"{"role":"assistant","content":"hello"}"#;
    let msg: Message = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "hello");
}

#[test]
fn test_role_display_name() {
    assert_eq!(Role::User.display_name(), "You");
    assert_eq!(Role::Assistant.display_name(), "AI");
    assert_eq!(Role::Developer.display_name(), "System");
    assert_eq!(Role::System.display_name(), "System");
}

#[test]
fn test_role_is_visible() {
    assert!(Role::User.is_visible());
    assert!(Role::Assistant.is_visible());
    assert!(!Role::Developer.is_visible());
    assert!(!Role::System.is_visible());
}

// ============================================================================
// CLI parsing tests
// ============================================================================

#[test]
fn test_parse_cli_empty_args() {
    let cli = parse_cli_args(&[]).expect("parse");
    assert_eq!(cli.mode, Mode::Chat);
    assert!(cli.prompt.is_none());
    assert!(cli.gf_files.is_empty());
    assert!(!cli.help);
    assert!(!cli.use_local);
    assert!(cli.model_override.is_none());
    assert!(cli.max_tokens.is_none());
    assert!(cli.system_override.is_none());
    assert!(cli.output.is_none());
    assert!(cli.directory.is_none());
    assert!(cli.extensions.is_empty());
}

#[test]
fn test_parse_cli_simple_prompt() {
    let cli = parse_cli_args(&args(&["hello", "world"])).expect("parse");
    assert_eq!(cli.mode, Mode::Chat);
    assert_eq!(cli.prompt.as_deref(), Some("hello world"));
}

#[test]
fn test_parse_cli_help_short() {
    let cli = parse_cli_args(&args(&["-h"])).expect("parse");
    assert!(cli.help);
}

#[test]
fn test_parse_cli_help_long() {
    let cli = parse_cli_args(&args(&["--help"])).expect("parse");
    assert!(cli.help);
}

#[test]
fn test_parse_cli_help_word() {
    let cli = parse_cli_args(&args(&["help"])).expect("parse");
    assert!(cli.help);
}

#[test]
fn test_parse_cli_model_override_short() {
    let cli = parse_cli_args(&args(&["-m", "gpt-4o", "hi"])).expect("parse");
    assert_eq!(cli.model_override.as_deref(), Some("gpt-4o"));
    assert_eq!(cli.prompt.as_deref(), Some("hi"));
}

#[test]
fn test_parse_cli_model_override_long() {
    let cli = parse_cli_args(&args(&["--model", "claude-3-5-sonnet"])).expect("parse");
    assert_eq!(cli.model_override.as_deref(), Some("claude-3-5-sonnet"));
}

#[test]
fn test_parse_cli_model_missing_value() {
    assert!(parse_cli_args(&args(&["--model"])).is_err());
}

#[test]
fn test_parse_cli_local_flag() {
    let cli = parse_cli_args(&args(&["--local", "hello"])).expect("parse");
    assert!(cli.use_local);
    assert_eq!(cli.prompt.as_deref(), Some("hello"));
}

#[test]
fn test_parse_cli_max_tokens_valid() {
    let cli = parse_cli_args(&args(&["--max-tokens", "1024", "hi"])).expect("parse");
    assert_eq!(cli.max_tokens, Some(1024));
}

#[test]
fn test_parse_cli_max_tokens_missing_value() {
    assert!(parse_cli_args(&args(&["--max-tokens"])).is_err());
}

#[test]
fn test_parse_cli_max_tokens_invalid_value() {
    assert!(parse_cli_args(&args(&["--max-tokens", "not-a-number"])).is_err());
}

#[test]
fn test_parse_cli_system_override_short() {
    let cli = parse_cli_args(&args(&["-s", "you are terse", "hello"])).expect("parse");
    assert_eq!(cli.system_override.as_deref(), Some("you are terse"));
    assert_eq!(cli.prompt.as_deref(), Some("hello"));
}

#[test]
fn test_parse_cli_system_override_long() {
    let cli = parse_cli_args(&args(&["--system", "be brief"])).expect("parse");
    assert_eq!(cli.system_override.as_deref(), Some("be brief"));
}

#[test]
fn test_parse_cli_system_missing_value() {
    assert!(parse_cli_args(&args(&["--system"])).is_err());
}

#[test]
fn test_parse_cli_output_short() {
    let cli = parse_cli_args(&args(&["-o", "out.md", "doc"])).expect("parse");
    assert_eq!(cli.output.as_deref(), Some("out.md"));
    assert_eq!(cli.mode, Mode::Doc);
}

#[test]
fn test_parse_cli_output_long() {
    let cli = parse_cli_args(&args(&["--output", "out.md"])).expect("parse");
    assert_eq!(cli.output.as_deref(), Some("out.md"));
}

#[test]
fn test_parse_cli_output_missing_value() {
    assert!(parse_cli_args(&args(&["--output"])).is_err());
}

#[test]
fn test_parse_cli_doc_subcommand() {
    let cli = parse_cli_args(&args(&["doc"])).expect("parse");
    assert_eq!(cli.mode, Mode::Doc);
}

#[test]
fn test_parse_cli_readme_subcommand() {
    let cli = parse_cli_args(&args(&["readme", "src/"])).expect("parse");
    assert_eq!(cli.mode, Mode::Readme);
    assert_eq!(cli.directory.as_deref(), Some("src/"));
}

#[test]
fn test_parse_cli_readme_with_extensions() {
    let cli = parse_cli_args(&args(&["readme", "src/", "rs", "toml"])).expect("parse");
    assert_eq!(cli.mode, Mode::Readme);
    assert_eq!(cli.directory.as_deref(), Some("src/"));
    assert_eq!(cli.extensions, vec![s("rs"), s("toml")]);
}

#[test]
fn test_parse_cli_doc_not_first_positional_is_prompt() {
    // "doc" only triggers subcommand mode when it's the first positional
    let cli = parse_cli_args(&args(&["explain", "doc"])).expect("parse");
    assert_eq!(cli.mode, Mode::Chat);
    assert_eq!(cli.prompt.as_deref(), Some("explain doc"));
}

#[test]
fn test_parse_cli_readme_not_first_positional_is_prompt() {
    let cli = parse_cli_args(&args(&["write", "readme"])).expect("parse");
    assert_eq!(cli.mode, Mode::Chat);
    assert_eq!(cli.prompt.as_deref(), Some("write readme"));
}

#[test]
fn test_parse_cli_gf_with_existing_files() {
    let dir = TmpDir::new("gf-parse");
    let f1 = dir.write("a.txt", "alpha");
    let f2 = dir.write("b.txt", "beta");

    let cli = parse_cli_args(&args(&[
        "--gf",
        &path_string(&f1),
        &path_string(&f2),
        "explain these",
    ]))
    .expect("parse");

    assert_eq!(cli.gf_files.len(), 2);
    assert_eq!(cli.prompt.as_deref(), Some("explain these"));
}

#[test]
fn test_parse_cli_gf_stops_at_nonfile() {
    // The --gf slurp should stop at the first non-file argument
    // (so trailing positional prompts work).
    let dir = TmpDir::new("gf-stop");
    let f1 = dir.write("real.txt", "hi");

    let cli = parse_cli_args(&args(&[
        "--gf",
        &path_string(&f1),
        "this-is-not-a-file",
        "and-this-also",
    ]))
    .expect("parse");

    assert_eq!(cli.gf_files.len(), 1);
    assert_eq!(cli.prompt.as_deref(), Some("this-is-not-a-file and-this-also"));
}

#[test]
fn test_parse_cli_gf_stops_at_flag() {
    let dir = TmpDir::new("gf-flag-stop");
    let f1 = dir.write("c.txt", "x");

    let cli = parse_cli_args(&args(&[
        "--gf",
        &path_string(&f1),
        "-m",
        "gpt-4o",
        "go",
    ]))
    .expect("parse");

    assert_eq!(cli.gf_files.len(), 1);
    assert_eq!(cli.model_override.as_deref(), Some("gpt-4o"));
    assert_eq!(cli.prompt.as_deref(), Some("go"));
}

#[test]
fn test_parse_cli_gf_empty_collection() {
    // --gf followed immediately by non-file means no files were collected.
    let cli = parse_cli_args(&args(&["--gf", "this-file-does-not-exist", "hi"])).expect("parse");
    assert!(cli.gf_files.is_empty());
    assert_eq!(cli.prompt.as_deref(), Some("this-file-does-not-exist hi"));
}

#[test]
fn test_parse_cli_combined_flags() {
    let cli = parse_cli_args(&args(&[
        "-m",
        "gpt-4o",
        "--max-tokens",
        "512",
        "-s",
        "be terse",
        "-o",
        "out.md",
        "doc",
    ]))
    .expect("parse");

    assert_eq!(cli.model_override.as_deref(), Some("gpt-4o"));
    assert_eq!(cli.max_tokens, Some(512));
    assert_eq!(cli.system_override.as_deref(), Some("be terse"));
    assert_eq!(cli.output.as_deref(), Some("out.md"));
    assert_eq!(cli.mode, Mode::Doc);
}

#[test]
fn test_parse_cli_doc_with_prompt() {
    let cli = parse_cli_args(&args(&["doc", "make", "it", "concise"])).expect("parse");
    assert_eq!(cli.mode, Mode::Doc);
    assert_eq!(cli.prompt.as_deref(), Some("make it concise"));
}

#[test]
fn test_parse_cli_readme_directory_required() {
    // The parser doesn't enforce this — run_readme does. We just verify
    // that omitting a directory leaves the field None.
    let cli = parse_cli_args(&args(&["readme"])).expect("parse");
    assert_eq!(cli.mode, Mode::Readme);
    assert!(cli.directory.is_none());
}

// ============================================================================
// load_files tests
// ============================================================================

#[test]
fn test_load_files_existing_file() {
    let dir = TmpDir::new("load-existing");
    let f = dir.write("file.txt", "the content");

    let mut ctx = ConversationContext::new("gpt-4o");
    let added = load_files(&mut ctx, &[path_string(&f)]);

    assert_eq!(added.len(), 1);
    assert_eq!(ctx.input.len(), 1);
    assert_eq!(ctx.input[0].role, Role::User);
    assert!(ctx.input[0].content.contains("the content"));
    assert!(ctx.input[0].content.contains(":::"));
}

#[test]
fn test_load_files_missing_file_skipped() {
    let mut ctx = ConversationContext::new("gpt-4o");
    let added = load_files(&mut ctx, &[s("/this/path/should/not/exist.txt")]);

    assert!(added.is_empty());
    assert!(ctx.input.is_empty());
}

#[test]
fn test_load_files_mixed_existing_and_missing() {
    let dir = TmpDir::new("load-mixed");
    let f = dir.write("real.txt", "real");

    let mut ctx = ConversationContext::new("gpt-4o");
    let added = load_files(
        &mut ctx,
        &[s("/no/such/file.txt"), path_string(&f)],
    );

    assert_eq!(added.len(), 1);
    assert_eq!(ctx.input.len(), 1);
}

#[test]
fn test_load_files_empty_paths() {
    let mut ctx = ConversationContext::new("gpt-4o");
    let added = load_files(&mut ctx, &[]);
    assert!(added.is_empty());
    assert!(ctx.input.is_empty());
}

#[test]
fn test_load_files_trims_whitespace() {
    let dir = TmpDir::new("load-trim");
    let f = dir.write("trim.txt", "x");

    let path_with_ws = format!("  {}  ", path_string(&f));
    let mut ctx = ConversationContext::new("gpt-4o");
    let added = load_files(&mut ctx, &[path_with_ws]);

    assert_eq!(added.len(), 1);
    assert_eq!(added[0], path_string(&f));
}

#[test]
fn test_load_files_appends_to_existing_context() {
    let dir = TmpDir::new("load-append");
    let f = dir.write("append.txt", "appended");

    let mut ctx = ConversationContext::new("gpt-4o");
    ctx.input.push(Message {
        role: Role::Developer,
        content: "system msg".to_string(),
    });
    load_files(&mut ctx, &[path_string(&f)]);

    assert_eq!(ctx.input.len(), 2);
    assert_eq!(ctx.input[0].role, Role::Developer);
    assert_eq!(ctx.input[1].role, Role::User);
}

// ============================================================================
// split_args tests
// ============================================================================

#[test]
fn test_split_args_empty() {
    assert!(split_args("").is_empty());
}

#[test]
fn test_split_args_single_word() {
    assert_eq!(split_args("foo"), vec![s("foo")]);
}

#[test]
fn test_split_args_multiple_words() {
    assert_eq!(split_args("foo bar baz"), vec![s("foo"), s("bar"), s("baz")]);
}

#[test]
fn test_split_args_double_quoted() {
    assert_eq!(
        split_args("foo \"bar baz\" qux"),
        vec![s("foo"), s("bar baz"), s("qux")]
    );
}

#[test]
fn test_split_args_single_quoted() {
    assert_eq!(
        split_args("foo 'bar baz' qux"),
        vec![s("foo"), s("bar baz"), s("qux")]
    );
}

#[test]
fn test_split_args_mixed_quotes() {
    assert_eq!(
        split_args("'one two' \"three four\" five"),
        vec![s("one two"), s("three four"), s("five")]
    );
}

#[test]
fn test_split_args_extra_whitespace() {
    assert_eq!(split_args("   foo    bar   "), vec![s("foo"), s("bar")]);
}

#[test]
fn test_split_args_path_with_spaces() {
    assert_eq!(
        split_args("\"/path with spaces/file.txt\""),
        vec![s("/path with spaces/file.txt")]
    );
}

// ============================================================================
// min_distance / find_matching_word tests
// ============================================================================

#[test]
fn test_min_distance_identical() {
    assert_eq!(min_distance("hello", "hello"), 0);
}

#[test]
fn test_min_distance_empty() {
    assert_eq!(min_distance("", ""), 0);
    assert_eq!(min_distance("abc", ""), 3);
    assert_eq!(min_distance("", "abc"), 3);
}

#[test]
fn test_min_distance_single_substitution() {
    assert_eq!(min_distance("cat", "bat"), 1);
}

#[test]
fn test_min_distance_insert_delete() {
    assert_eq!(min_distance("kitten", "sitting"), 3);
}

#[test]
fn test_find_matching_word_basic() {
    let words = vec![s("help"), s("clear"), s("quit"), s("yank")];
    let result = find_matching_word("hep", &words).expect("found");
    assert_eq!(result, "help");
}

#[test]
fn test_find_matching_word_empty_list() {
    let words: Vec<String> = vec![];
    assert!(find_matching_word("anything", &words).is_err());
}

#[test]
fn test_find_matching_word_exact() {
    let words = vec![s("help"), s("clear")];
    let result = find_matching_word("help", &words).expect("found");
    assert_eq!(result, "help");
}

// ============================================================================
// walk_directory tests
// ============================================================================

#[test]
fn test_walk_directory_finds_files() {
    let dir = TmpDir::new("walk-find");
    dir.write("a.rs", "fn main() {}");
    dir.write("b.rs", "// b");
    dir.write("c.txt", "text");

    let exts = HashSet::from(["rs"]);
    let excluded = HashSet::new();
    let results = walk_directory(dir.path_str(), &exts, &excluded).expect("walk");

    assert_eq!(results.len(), 2);
    let contents: Vec<&str> = results.iter().map(|(_, c)| c.as_str()).collect();
    assert!(contents.contains(&"fn main() {}"));
    assert!(contents.contains(&"// b"));
}

#[test]
fn test_walk_directory_excludes_directory() {
    let dir = TmpDir::new("walk-excl");
    dir.write("keep.rs", "keep");
    dir.write("target/skip.rs", "skip");

    let exts = HashSet::new();
    let excluded = HashSet::from(["target"]);
    let results = walk_directory(dir.path_str(), &exts, &excluded).expect("walk");
    let contents: Vec<&str> = results.iter().map(|(_, c)| c.as_str()).collect();

    assert!(contents.contains(&"keep"));
    assert!(!contents.contains(&"skip"));
}

#[test]
fn test_walk_directory_no_extension_filter() {
    let dir = TmpDir::new("walk-noext");
    dir.write("a.rs", "rust");
    dir.write("b.toml", "toml");
    dir.write("c.md", "md");

    let exts = HashSet::new();
    let excluded = HashSet::new();
    let results = walk_directory(dir.path_str(), &exts, &excluded).expect("walk");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_walk_directory_skips_hidden_files() {
    let dir = TmpDir::new("walk-hidden");
    dir.write("visible.rs", "v");
    dir.write(".hidden.rs", "h");

    let exts = HashSet::new();
    let excluded = HashSet::new();
    let results = walk_directory(dir.path_str(), &exts, &excluded).expect("walk");
    let contents: Vec<&str> = results.iter().map(|(_, c)| c.as_str()).collect();

    assert!(contents.contains(&"v"));
    assert!(!contents.contains(&"h"));
}

#[test]
fn test_walk_directory_empty_dir() {
    let dir = TmpDir::new("walk-empty");
    let exts = HashSet::new();
    let excluded = HashSet::new();
    let results = walk_directory(dir.path_str(), &exts, &excluded).expect("walk");
    assert!(results.is_empty());
}

#[test]
fn test_walk_directory_nonexistent_path() {
    let exts = HashSet::new();
    let excluded = HashSet::new();
    let results = walk_directory("/no/such/dir/here", &exts, &excluded).expect("walk");
    assert!(results.is_empty());
}

// ============================================================================
// sequence_equals tests
// ============================================================================

#[test]
fn test_sequence_equals_same_order() {
    let a = vec![s("a"), s("b"), s("c")];
    let b = vec![s("a"), s("b"), s("c")];
    assert!(sequence_equals(&a, &b));
}

#[test]
fn test_sequence_equals_different_order() {
    let a = vec![s("a"), s("b"), s("c")];
    let b = vec![s("c"), s("b"), s("a")];
    assert!(sequence_equals(&a, &b));
}

#[test]
fn test_sequence_equals_different_length() {
    let a = vec![s("a"), s("b")];
    let b = vec![s("a"), s("b"), s("c")];
    assert!(!sequence_equals(&a, &b));
}

#[test]
fn test_sequence_equals_different_items() {
    let a = vec![s("a"), s("b")];
    let b = vec![s("a"), s("c")];
    assert!(!sequence_equals(&a, &b));
}

#[test]
fn test_sequence_equals_both_empty() {
    let a: Vec<String> = vec![];
    let b: Vec<String> = vec![];
    assert!(sequence_equals(&a, &b));
}

// ============================================================================
// get_default_model tests
// ============================================================================

#[test]
fn test_get_default_model_anthropic_prefers_sonnet() {
    let models = vec![s("claude-3-opus"), s("claude-3-5-sonnet"), s("gpt-4o")];
    assert_eq!(get_default_model(&models, true), "claude-3-5-sonnet");
}

#[test]
fn test_get_default_model_no_anthropic_prefers_gpt() {
    let models = vec![s("claude-3-5-sonnet"), s("gpt-4o"), s("o1")];
    assert_eq!(get_default_model(&models, false), "gpt-4o");
}

#[test]
fn test_get_default_model_anthropic_no_sonnet_falls_back() {
    let models = vec![s("claude-3-opus"), s("gpt-4o")];
    // No sonnet → falls through to gpt logic
    assert_eq!(get_default_model(&models, true), "gpt-4o");
}

#[test]
fn test_get_default_model_first_fallback() {
    let models = vec![s("local/qwen3.5"), s("local/llama-3")];
    assert_eq!(get_default_model(&models, false), "local/qwen3.5");
}

#[test]
fn test_get_default_model_empty_list() {
    let models: Vec<String> = vec![];
    assert_eq!(get_default_model(&models, false), "default_model_name");
}

// ============================================================================
// filter_models tests
// ============================================================================

#[test]
fn test_filter_models_excludes_finetuned() {
    let models = vec![s("gpt-4o"), s("ft:gpt-4:my-org::abc123")];
    assert_eq!(filter_models(models), vec![s("gpt-4o")]);
}

#[test]
fn test_filter_models_excludes_old_openai() {
    let models = vec![
        s("ada"),
        s("babbage"),
        s("curie"),
        s("davinci"),
        s("text-davinci-003"),
        s("gpt-3.5-turbo"),
        s("gpt-4o"),
    ];
    assert_eq!(filter_models(models), vec![s("gpt-4o")]);
}

#[test]
fn test_filter_models_excludes_specialized() {
    let models = vec![
        s("whisper-1"),
        s("dall-e-3"),
        s("tts-1"),
        s("gpt-4o-realtime"),
        s("gpt-4o-audio-preview"),
        s("gpt-4o-mini"),
        s("gpt-4o"),
        s("claude-3-5-sonnet"),
    ];
    assert_eq!(filter_models(models), vec![s("gpt-4o"), s("claude-3-5-sonnet")]);
}

#[test]
fn test_filter_models_keeps_general_chat() {
    let models = vec![
        s("gpt-4o"),
        s("gpt-4"),
        s("claude-3-5-sonnet"),
        s("claude-3-opus"),
        s("o1"),
    ];
    assert_eq!(filter_models(models.clone()), models);
}

#[test]
fn test_filter_models_excludes_codex() {
    let models = vec![s("code-davinci-002"), s("gpt-4o-codex"), s("gpt-4o")];
    assert_eq!(filter_models(models), vec![s("gpt-4o")]);
}

// ============================================================================
// get_base_model_name / has_date_suffix tests
// ============================================================================

#[test]
fn test_get_base_model_name_with_date() {
    assert_eq!(get_base_model_name("claude-3-5-sonnet-20241022"), "claude-3-5-sonnet");
}

#[test]
fn test_get_base_model_name_without_date() {
    assert_eq!(get_base_model_name("gpt-4o"), "gpt-4o");
}

#[test]
fn test_get_base_model_name_short() {
    assert_eq!(get_base_model_name("o1"), "o1");
}

#[test]
fn test_has_date_suffix_yyyymmdd() {
    assert!(has_date_suffix("claude-3-5-sonnet-20241022"));
}

#[test]
fn test_has_date_suffix_no_date() {
    assert!(!has_date_suffix("gpt-4o"));
    assert!(!has_date_suffix("claude-3-5-sonnet"));
}

#[test]
fn test_has_date_suffix_too_short() {
    assert!(!has_date_suffix("foo-2024"));
}

// ============================================================================
// deduplicate_models tests
// ============================================================================

#[test]
fn test_deduplicate_models_prefers_dateless() {
    let models = vec![s("claude-3-5-sonnet"), s("claude-3-5-sonnet-20241022")];
    let dedup = deduplicate_models(models);
    assert_eq!(dedup.len(), 1);
    assert_eq!(dedup[0], "claude-3-5-sonnet");
}

#[test]
fn test_deduplicate_models_keeps_distinct_bases() {
    let models = vec![s("gpt-4o"), s("claude-3-5-sonnet"), s("o1")];
    let dedup = deduplicate_models(models);
    assert_eq!(dedup.len(), 3);
}

#[test]
fn test_deduplicate_models_all_dated_keeps_one() {
    let models = vec![
        s("claude-3-5-sonnet-20241022"),
        s("claude-3-5-sonnet-20240620"),
    ];
    let dedup = deduplicate_models(models);
    assert_eq!(dedup.len(), 1);
}

// ============================================================================
// parse_color tests
// ============================================================================

#[test]
fn test_parse_color_known_names() {
    assert_eq!(parse_color("red"), parse_color("RED"));
    assert_eq!(parse_color("green"), parse_color("Green"));
    assert_eq!(parse_color("yellow"), parse_color("YELLOW"));
    assert_eq!(parse_color("blue"), parse_color("blue"));
}

#[test]
fn test_parse_color_distinct_colors() {
    assert_ne!(parse_color("red"), parse_color("blue"));
    assert_ne!(parse_color("green"), parse_color("yellow"));
}

#[test]
fn test_parse_color_unknown_returns_reset() {
    // Reset is the default for unknown values; two unknowns should equal each other.
    assert_eq!(parse_color("nonsense"), parse_color("also-nonsense"));
}

#[test]
fn test_parse_color_dark_variants_distinct() {
    assert_ne!(parse_color("red"), parse_color("dark_red"));
    assert_ne!(parse_color("blue"), parse_color("dark_blue"));
}

#[test]
fn test_tmp_dir_path_accessor() {
    let dir = TmpDir::new("smoke");
    assert!(dir.path().exists());
    assert!(dir.path_str().contains("tc-tests-smoke"));
}
