use gwatch::git_engine::GitEngine;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git email");

    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git name");
}

fn git_add_commit(dir: &Path, message: &str) {
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .expect("Failed to git add");

    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .output()
        .expect("Failed to git commit");
}

fn run_git_command(dir: &Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to run git command");
}

fn create_test_repo() -> (GitEngine, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Canonicalize to avoid macOS /var vs /private/var issues
    let repo_path = temp_dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize temp dir");
    init_git_repo(&repo_path);
    let engine = GitEngine::new(&repo_path).expect("Failed to create GitEngine");
    (engine, temp_dir)
}

#[test]
fn test_temp_repo_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

    let git_dir = repo_path.join(".git");
    assert!(git_dir.exists(), ".git directory should exist");
}

#[test]
fn test_file_modification_workflow() {
    let (_engine, temp_dir) = create_test_repo();
    let repo_path = temp_dir.path();

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to modify file");

    let content = fs::read_to_string(&test_file).expect("Failed to read file");
    assert!(content.contains("line 3"));
}

#[test]
fn test_file_deletion_workflow() {
    let (_engine, temp_dir) = create_test_repo();
    let repo_path = temp_dir.path();

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    fs::write(&test_file, "line 1\nline 3\n").expect("Failed to modify file");

    let content = fs::read_to_string(&test_file).expect("Failed to read file");
    assert!(!content.contains("line 2"));
}

#[test]
fn test_new_file_workflow() {
    let (_engine, temp_dir) = create_test_repo();
    let repo_path = temp_dir.path();

    let initial_file = repo_path.join("initial.txt");
    fs::write(&initial_file, "initial\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    let new_file = repo_path.join("new_file.txt");
    fs::write(&new_file, "new content\n").expect("Failed to write new file");

    assert!(new_file.exists());
    assert!(initial_file.exists());
}

#[test]
fn test_staged_diff_detection() {
    let (_engine, temp_dir) = create_test_repo();
    let repo_path = temp_dir.path();

    // Create and commit initial file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    // Modify and stage
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to modify");
    run_git_command(repo_path, &["add", "test.txt"]);

    // Staged changes should exist
    let staged_output = Command::new("git")
        .args(["diff", "--cached", "--stat"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get staged diff");

    let output = String::from_utf8_lossy(&staged_output.stdout);
    assert!(
        output.contains("test.txt"),
        "Staged changes should include test.txt"
    );
}

#[test]
fn test_unstaged_diff_detection() {
    let (_engine, temp_dir) = create_test_repo();
    let repo_path = temp_dir.path();

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    // Stage a change
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to modify");
    run_git_command(repo_path, &["add", "test.txt"]);

    // Make another unstaged change
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to modify again");

    // Unstaged changes should show line 3
    let unstaged_output = Command::new("git")
        .args(["diff", "--stat"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get unstaged diff");

    let output = String::from_utf8_lossy(&unstaged_output.stdout);
    assert!(
        output.contains("test.txt"),
        "Unstaged changes should include test.txt"
    );
}

#[test]
fn test_relative_path() {
    let (engine, _temp) = create_test_repo();
    let full_path = engine.repo_root().join("test.txt");
    let relative = engine.relative_path(&full_path);
    assert_eq!(relative, "test.txt");
}

#[test]
fn test_relative_path_nested() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let nested = repo_path.join("src").join("lib.rs");
    fs::create_dir_all(repo_path.join("src")).unwrap();
    fs::write(&nested, "fn main() {}").unwrap();

    let relative = engine.relative_path(&nested);
    assert_eq!(relative, "src/lib.rs");
}

#[test]
fn test_diff_empty_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("empty.txt");

    fs::write(&file_path, "").unwrap();
    run_git_command(&repo_path, &["add", "empty.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "add empty"]);

    fs::write(&file_path, "content").unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert_eq!(diff.stats.added_count, 1);
}

#[test]
fn test_diff_binary_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("image.png");

    // Write binary content (explicit \0 byte for detection)
    fs::write(&file_path, [0, 1, 2, 3, 4, 5]).unwrap();
    run_git_command(&repo_path, &["add", "image.png"]);
    run_git_command(&repo_path, &["commit", "-m", "add image"]);

    // Modify it
    fs::write(&file_path, [0, 1, 2, 3, 4, 5, 6, 7]).unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert!(diff.is_binary);
}

#[test]
fn test_staged_diff_no_changes() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("test.txt");

    fs::write(&file_path, "content").unwrap();
    run_git_command(&repo_path, &["add", "test.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "commit"]);

    // No staged changes
    let diff = engine.compute_staged_diff(&file_path).unwrap();
    assert!(diff.hunks.is_empty());
}

#[test]
fn test_repo_root_is_absolute() {
    let (engine, _temp) = create_test_repo();
    assert!(engine.repo_root().is_absolute());
}

#[test]
fn test_diff_multiline_changes() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("multi.txt");

    fs::write(&file_path, "line1\nline2\nline3\n").unwrap();
    run_git_command(&repo_path, &["add", "multi.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    fs::write(&file_path, "line1\nmodified\nline3\nnew_line\n").unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert!(diff.stats.added_count >= 2);
    assert!(diff.stats.deleted_count >= 1);
}

// === Additional tests for coverage ===

#[test]
fn test_diff_deleted_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("to_delete.txt");

    fs::write(&file_path, "content\n").unwrap();
    run_git_command(&repo_path, &["add", "to_delete.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "add file"]);

    // Delete the file
    fs::remove_file(&file_path).unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert!(diff.is_deleted);
}

#[test]
fn test_diff_new_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    // Need an initial commit first
    let initial = repo_path.join("initial.txt");
    fs::write(&initial, "init\n").unwrap();
    run_git_command(&repo_path, &["add", "initial.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    // Create new untracked file
    let new_file = repo_path.join("brand_new.txt");
    fs::write(&new_file, "new content\n").unwrap();

    let diff = engine.compute_diff(&new_file).unwrap();
    assert!(diff.is_new_file);
    assert_eq!(diff.stats.added_count, 1);
}

#[test]
fn test_staged_diff_new_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    // Need initial commit
    let initial = repo_path.join("initial.txt");
    fs::write(&initial, "init\n").unwrap();
    run_git_command(&repo_path, &["add", "initial.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    // Create and stage new file
    let new_file = repo_path.join("staged_new.txt");
    fs::write(&new_file, "staged content\n").unwrap();
    run_git_command(&repo_path, &["add", "staged_new.txt"]);

    let diff = engine.compute_staged_diff(&new_file).unwrap();
    assert!(diff.is_new_file);
}

#[test]
fn test_staged_diff_deleted_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    let file_path = repo_path.join("to_stage_delete.txt");
    fs::write(&file_path, "content\n").unwrap();
    run_git_command(&repo_path, &["add", "to_stage_delete.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "add file"]);

    // Stage deletion
    run_git_command(&repo_path, &["rm", "to_stage_delete.txt"]);

    let diff = engine.compute_staged_diff(&file_path).unwrap();
    assert!(diff.is_deleted);
}

#[test]
fn test_unstaged_diff_deleted_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    let file_path = repo_path.join("unstaged_delete.txt");
    fs::write(&file_path, "content\n").unwrap();
    run_git_command(&repo_path, &["add", "unstaged_delete.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "add file"]);

    // Delete without staging
    fs::remove_file(&file_path).unwrap();

    let diff = engine.compute_unstaged_diff(&file_path).unwrap();
    assert!(diff.is_deleted);
}

#[test]
fn test_unstaged_diff_binary_file() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    let file_path = repo_path.join("binary.bin");
    fs::write(&file_path, [0, 1, 2, 3]).unwrap();
    run_git_command(&repo_path, &["add", "binary.bin"]);
    run_git_command(&repo_path, &["commit", "-m", "add binary"]);

    // Modify binary
    fs::write(&file_path, [0, 1, 2, 3, 4, 5]).unwrap();

    let diff = engine.compute_unstaged_diff(&file_path).unwrap();
    assert!(diff.is_binary);
}

#[test]
fn test_unstaged_diff_new_file_not_in_index() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    // Initial commit
    let initial = repo_path.join("initial.txt");
    fs::write(&initial, "init\n").unwrap();
    run_git_command(&repo_path, &["add", "initial.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    // New file not staged
    let new_file = repo_path.join("untracked.txt");
    fs::write(&new_file, "untracked content\n").unwrap();

    let diff = engine.compute_unstaged_diff(&new_file).unwrap();
    assert!(diff.is_new_file);
}

#[test]
fn test_compute_diff_context_lines() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("context.txt");

    // Create file with many lines
    let original: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    fs::write(&file_path, &original).unwrap();
    run_git_command(&repo_path, &["add", "context.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    // Modify one line in the middle
    let modified = original.replace("line 10", "MODIFIED");
    fs::write(&file_path, &modified).unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert!(!diff.hunks.is_empty());
    // Should have context lines around the change
    assert!(diff.hunks[0].lines.len() > 2);
}

#[test]
fn test_staged_diff_both_none() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");

    // Initial commit
    let initial = repo_path.join("initial.txt");
    fs::write(&initial, "init\n").unwrap();
    run_git_command(&repo_path, &["add", "initial.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    // Query staged diff for non-existent file
    let nonexistent = repo_path.join("does_not_exist.txt");
    let diff = engine.compute_staged_diff(&nonexistent).unwrap();
    assert!(diff.hunks.is_empty());
    assert!(!diff.is_new_file);
    assert!(!diff.is_deleted);
}

#[test]
fn test_diff_hunk_line_numbers() {
    let (engine, temp) = create_test_repo();
    let repo_path = temp
        .path()
        .canonicalize()
        .expect("Failed to canonicalize repo path");
    let file_path = repo_path.join("numbered.txt");

    fs::write(&file_path, "a\nb\nc\n").unwrap();
    run_git_command(&repo_path, &["add", "numbered.txt"]);
    run_git_command(&repo_path, &["commit", "-m", "initial"]);

    fs::write(&file_path, "a\nX\nc\n").unwrap();

    let diff = engine.compute_diff(&file_path).unwrap();
    assert!(!diff.hunks.is_empty());

    // Check that hunks have valid line numbers
    let hunk = &diff.hunks[0];
    assert!(hunk.old_start > 0);
    assert!(hunk.new_start > 0);
}
