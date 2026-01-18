use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
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

fn git_add_commit(dir: &std::path::Path, message: &str) {
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
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to modify file");

    let content = fs::read_to_string(&test_file).expect("Failed to read file");
    assert!(content.contains("line 3"));
}

#[test]
fn test_file_deletion_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\nline 2\nline 3\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    fs::write(&test_file, "line 1\nline 3\n").expect("Failed to modify file");

    let content = fs::read_to_string(&test_file).expect("Failed to read file");
    assert!(!content.contains("line 2"));
}

#[test]
fn test_new_file_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

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
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

    // Create and commit initial file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    // Modify and stage
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to modify");
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage");

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
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    init_git_repo(repo_path);

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "line 1\n").expect("Failed to write file");
    git_add_commit(repo_path, "Initial commit");

    // Stage a change
    fs::write(&test_file, "line 1\nline 2\n").expect("Failed to modify");
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage");

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
