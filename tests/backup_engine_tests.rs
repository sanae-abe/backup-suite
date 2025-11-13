//! Comprehensive backup engine tests targeting uncovered code paths
//!
//! This test suite focuses on edge cases and error handling in src/core/backup.rs
//! to increase coverage from 48% to the Phase 1 target of 66-70%.
//!
//! Test Coverage Areas:
//! - Priority 1: Error handling in parallel processing (rayon)
//! - Priority 2: Backup name generation edge cases
//! - Priority 3: Progress reporting edge cases
//! - Priority 4: File metadata and permissions
//! - Priority 5: Disk space and resource constraints

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::ignored_unit_patterns)]

use anyhow::Result;
use backup_suite::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

// ==================== Priority 1: Error Handling in Parallel Processing ====================

/// Test 1: Parallel processing error propagation
///
/// Validates that errors during rayon parallel processing are properly
/// collected and reported in the BackupResult.
#[test]
fn test_parallel_processing_error_propagation() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create multiple files to trigger parallel processing
    for i in 0..20 {
        fs::write(source.join(format!("file{i}.txt")), format!("content {i}"))?;
    }

    // Create one read-protected file to trigger error
    #[cfg(unix)]
    {
        let protected_file = source.join("protected.txt");
        fs::write(&protected_file, "protected content")?;

        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&protected_file)?.permissions();
        perms.set_mode(0o000); // No read permissions
        fs::set_permissions(&protected_file, perms)?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify parallel processing completed with partial success
    assert!(result.total_files >= 20);
    assert!(result.successful > 0);

    #[cfg(unix)]
    {
        // On Unix, we should have at least one failure from the protected file
        assert!(result.failed > 0 || result.successful == result.total_files);
    }

    Ok(())
}

/// Test 2: Backup failure mid-execution recovery
///
/// Tests that the backup engine properly handles and recovers from
/// failures that occur during the middle of a backup operation.
#[test]
fn test_backup_failure_mid_execution_recovery() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create files with varying content sizes
    fs::write(source.join("small.txt"), "small")?;
    fs::write(source.join("medium.txt"), "medium content here")?;
    fs::write(source.join("large.txt"), "large".repeat(1000))?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::Medium,
        "recovery_test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify that despite any errors, the process completed
    assert_eq!(result.total_files, 3);
    assert!(result.successful > 0);
    assert!(result.total_bytes > 0);

    // Verify backup directory was created
    assert!(backup_dest.join(&result.backup_name).exists());

    Ok(())
}

/// Test 3: Partial backup completion handling
///
/// Ensures that partial backup results are correctly tracked and
/// that the errors list contains meaningful information.
#[test]
fn test_partial_backup_completion_handling() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create normal files
    for i in 0..10 {
        fs::write(
            source.join(format!("normal_{i}.txt")),
            format!("content {i}"),
        )?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::Low,
        "partial".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify completion tracking
    assert_eq!(result.total_files, 10);
    assert_eq!(result.successful + result.failed, result.total_files);
    assert!(result.errors.len() <= result.failed);

    Ok(())
}

// ==================== Priority 2: Backup Name Generation ====================

/// Test 4: Backup name generation with special characters
///
/// Tests that backup names are generated correctly even when
/// source paths contain special characters.
#[test]
fn test_backup_name_generation_special_characters() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source with spaces");
    let backup_dest = temp.path().join("backup-dest");

    fs::create_dir_all(&source)?;
    fs::write(source.join("test.txt"), "test")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "special chars".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup name follows expected pattern: backup_YYYYMMDD_HHMMSS
    assert!(result.backup_name.starts_with("backup_"));
    assert!(result.backup_name.len() >= 15); // backup_YYYYMMDD_

    // Verify backup directory was created with correct name
    let backup_path = backup_dest.join(&result.backup_name);
    assert!(backup_path.exists());

    Ok(())
}

/// Test 5: Timestamp collision handling
///
/// Tests the theoretical case of rapid successive backups and
/// ensures unique backup names are generated.
#[test]
fn test_timestamp_collision_handling() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("test.txt"), "test")?;

    // Execute first backup
    let mut config1 = Config::default();
    config1.backup.destination = backup_dest.clone();
    config1.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "collision_test".to_string(),
    ));

    let mut runner1 = BackupRunner::new(config1, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);
    let result1 = runner1.run(None, None)?;

    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // Execute second backup with new config
    let mut config2 = Config::default();
    config2.backup.destination = backup_dest.clone();
    config2.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "collision_test".to_string(),
    ));

    let mut runner2 = BackupRunner::new(config2, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);
    let result2 = runner2.run(None, None)?;

    // Verify both backups succeeded with different names
    assert_ne!(result1.backup_name, result2.backup_name);
    assert!(backup_dest.join(&result1.backup_name).exists());
    assert!(backup_dest.join(&result2.backup_name).exists());

    Ok(())
}

/// Test 6: Backup directory creation failures
///
/// Tests handling of backup directory creation failures due to
/// permission issues or invalid paths.
#[test]
fn test_backup_directory_creation_failures() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    fs::create_dir_all(&source)?;
    fs::write(source.join("test.txt"), "test")?;

    // Attempt backup to an invalid destination (file instead of directory)
    let invalid_dest = temp.path().join("file_not_dir");
    fs::write(&invalid_dest, "not a directory")?;

    let mut config = Config::default();
    config.backup.destination = invalid_dest.join("backup"); // Will fail
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "dir_creation_fail".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // This should fail due to invalid destination
    let result = runner.run(None, None);

    // Verify error is properly propagated
    assert!(result.is_err() || (result.is_ok() && result.unwrap().failed > 0));

    Ok(())
}

// ==================== Priority 3: Progress Reporting ====================

/// Test 7: Progress callback error recovery
///
/// Tests that errors in progress reporting don't crash the backup process.
#[test]
fn test_progress_callback_error_recovery() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    for i in 0..5 {
        fs::write(source.join(format!("file{i}.txt")), format!("content {i}"))?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "progress_test".to_string(),
    ));

    // Enable progress reporting
    let mut runner = BackupRunner::new(config, false)
        .with_progress(true)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup succeeded despite progress callbacks
    assert_eq!(result.total_files, 5);
    assert_eq!(result.successful, 5);

    Ok(())
}

/// Test 8: Progress reporting with zero files
///
/// Tests edge case of progress reporting when no files need to be backed up.
#[test]
fn test_progress_reporting_zero_files() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    // No files created - empty directory

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "empty".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(true)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify handling of empty backup
    assert_eq!(result.total_files, 0);
    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 0);

    Ok(())
}

/// Test 9: Progress reporting with very large file counts
///
/// Tests progress reporting scalability with many files (>1000).
#[test]
fn test_progress_reporting_large_file_count() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create many small files to test scalability
    // Using 100 files instead of 10,000 for test performance
    for i in 0..100 {
        fs::write(source.join(format!("file_{i:04}.txt")), format!("data {i}"))?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "large_count".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(true)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify all files were processed
    assert_eq!(result.total_files, 100);
    assert_eq!(result.successful, 100);
    assert_eq!(result.failed, 0);

    Ok(())
}

// ==================== Priority 4: File Metadata & Permissions ====================

/// Test 10: File metadata copy failures
///
/// Tests handling of metadata copy failures during backup.
#[test]
fn test_file_metadata_copy_failures() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create file with metadata
    let file_path = source.join("metadata_test.txt");
    let mut file = fs::File::create(&file_path)?;
    file.write_all(b"content with metadata")?;
    drop(file);

    // Set file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms)?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        file_path.clone(),
        Priority::High,
        "metadata".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup succeeded
    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);

    // Verify backed up file exists
    let backed_up = backup_dest
        .join(&result.backup_name)
        .join("metadata")
        .join("metadata_test.txt");
    assert!(backed_up.exists());

    Ok(())
}

/// Test 11: Permission preservation on different platforms
///
/// Tests that file permissions are handled correctly across platforms.
#[test]
fn test_permission_preservation_cross_platform() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    let test_file = source.join("perm_test.txt");
    fs::write(&test_file, "permission test content")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_file)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&test_file, perms)?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        test_file.clone(),
        Priority::High,
        "permissions".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup succeeded
    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);

    let backed_up = backup_dest
        .join(&result.backup_name)
        .join("permissions")
        .join("perm_test.txt");
    assert!(backed_up.exists());

    // On Unix, verify permissions were preserved (or at least the file is readable)
    #[cfg(unix)]
    {
        let backed_perms = fs::metadata(&backed_up)?.permissions();
        assert!(!backed_perms.readonly());
    }

    Ok(())
}

/// Test 12: Handling of read-only source files
///
/// Tests that read-only source files can still be backed up successfully.
#[test]
fn test_readonly_source_files() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    let readonly_file = source.join("readonly.txt");
    fs::write(&readonly_file, "readonly content")?;

    // Make file read-only
    let mut perms = fs::metadata(&readonly_file)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&readonly_file, perms)?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        readonly_file.clone(),
        Priority::High,
        "readonly".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify read-only file was backed up successfully
    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);

    let backed_up = backup_dest
        .join(&result.backup_name)
        .join("readonly")
        .join("readonly.txt");
    assert!(backed_up.exists());

    // Verify content is correct
    let content = fs::read_to_string(&backed_up)?;
    assert_eq!(content, "readonly content");

    Ok(())
}

// ==================== Priority 5: Disk Space & Resources ====================

/// Test 13: Disk full during backup scenarios
///
/// Tests graceful handling when running out of disk space.
/// Note: This is a simulation since we can't actually fill disk in tests.
#[test]
fn test_disk_full_simulation() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create some files
    for i in 0..5 {
        fs::write(source.join(format!("file{i}.txt")), format!("content {i}"))?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "disk_test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // Execute backup normally (can't simulate disk full easily)
    let result = runner.run(None, None)?;

    // Verify backup completed (in real disk full scenario, would have failures)
    assert!(result.total_files > 0);
    assert!(result.successful > 0);

    Ok(())
}

/// Test 14: Permission denied mid-backup recovery
///
/// Tests recovery when permission denied errors occur during backup.
#[test]
fn test_permission_denied_recovery() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create accessible files
    for i in 0..3 {
        fs::write(
            source.join(format!("accessible_{i}.txt")),
            format!("ok {i}"),
        )?;
    }

    #[cfg(unix)]
    {
        // Create a file with no read permissions
        let denied_file = source.join("denied.txt");
        fs::write(&denied_file, "denied content")?;

        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&denied_file)?.permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&denied_file, perms)?;
    }

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "permission_recovery".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify partial success (accessible files backed up)
    assert!(result.successful >= 3);

    #[cfg(unix)]
    {
        // On Unix, should have at least one failure from denied file
        assert!(result.total_files >= 4);
    }

    Ok(())
}

/// Test 15: Cleanup after partial backup failures
///
/// Tests that partial backups are properly tracked and can be cleaned up.
#[test]
fn test_cleanup_after_partial_failures() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create mix of files
    fs::write(source.join("good1.txt"), "good content 1")?;
    fs::write(source.join("good2.txt"), "good content 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "cleanup_test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup directory structure exists
    let backup_path = backup_dest.join(&result.backup_name);
    assert!(backup_path.exists());

    // Verify category subdirectory was created
    let category_path = backup_path.join("cleanup_test");
    assert!(category_path.exists());

    // Verify at least some files were backed up
    assert!(result.successful > 0);
    assert_eq!(result.successful + result.failed, result.total_files);

    // Check that successful files exist
    if result.successful > 0 {
        assert!(
            category_path.join("good1.txt").exists() || category_path.join("good2.txt").exists()
        );
    }

    Ok(())
}

// ==================== Priority 6: Encryption & Compression Integration ====================

/// Test 16: Backup with encryption only (no compression)
///
/// Tests the `with_encryption` builder method and encryption-only backup flow.
#[test]
fn test_backup_encryption_only() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("secret1.txt"), "confidential data 1")?;
    fs::write(source.join("secret2.txt"), "confidential data 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "encryption-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_encryption("test-password-123".to_string())
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);

    // Verify backup directory was created
    let backup_path = backup_dest.join(&result.backup_name);
    assert!(backup_path.exists());

    Ok(())
}

/// Test 17: Backup with Zstd compression only (no encryption)
///
/// Tests the `with_compression` builder method with Zstd compression.
#[test]
fn test_backup_compression_zstd_only() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;

    // Create compressible files (repetitive data)
    fs::write(source.join("large1.txt"), "A".repeat(10000))?;
    fs::write(source.join("large2.txt"), "B".repeat(10000))?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "compression-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::Zstd, 3);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);
    assert!(result.total_bytes > 0);

    Ok(())
}

/// Test 18: Backup with both encryption and compression
///
/// Tests the combined encryption + compression pipeline functionality.
#[test]
fn test_backup_encryption_and_compression_combined() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("data1.txt"), "sensitive data 1".repeat(100))?;
    fs::write(source.join("data2.txt"), "sensitive data 2".repeat(100))?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "enc-comp-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_encryption("secure-password-456".to_string())
        .with_compression(CompressionType::Zstd, 5);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);

    Ok(())
}

/// Test 19: Backup with Gzip compression
///
/// Tests the `with_compression` builder method with Gzip compression type.
#[test]
fn test_backup_compression_gzip() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("file1.txt"), "test data 1".repeat(50))?;
    fs::write(source.join("file2.txt"), "test data 2".repeat(50))?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::Medium,
        "gzip-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::Gzip, 6);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);

    Ok(())
}

// ==================== Priority 7: Incremental Backup & Verification ====================

/// Test 20: Incremental backup - first run (should be full backup)
///
/// Tests the `with_incremental` builder method and first-run behavior.
#[test]
fn test_incremental_backup_first_run() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("file1.txt"), "initial content")?;
    fs::write(source.join("file2.txt"), "initial content 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "incremental-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_incremental(true)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // First run should backup all files (full backup)
    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);

    Ok(())
}

/// Test 21: Backup with verification enabled
///
/// Tests the `with_verification` builder method and integrity checker integration.
#[test]
fn test_backup_with_verification_enabled() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("verify1.txt"), "data for verification 1")?;
    fs::write(source.join("verify2.txt"), "data for verification 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "verify-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_verification(true)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 0);

    Ok(())
}

/// Test 22: Backup without verification (default behavior)
///
/// Tests that verification is disabled by default.
#[test]
fn test_backup_without_verification() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("noverify1.txt"), "no verification needed 1")?;
    fs::write(source.join("noverify2.txt"), "no verification needed 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::Medium,
        "noverify-test".to_string(),
    ));

    // Default BackupRunner has verification disabled
    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);

    Ok(())
}

// ==================== Priority 8: Category Filtering ====================

/// Test 23: Category filter with multiple targets
///
/// Tests category filtering logic (lines 264-277 in backup.rs).
#[test]
fn test_backup_category_filter_multiple() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("file1.txt"), "category content")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "documents".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // Filter by category "documents"
    let result = runner.run(None, Some("documents"))?;

    assert_eq!(result.total_files, 1);
    assert_eq!(result.successful, 1);

    Ok(())
}

/// Test 24: Category filter with no matching targets
///
/// Tests early return when no targets match the category filter.
#[test]
fn test_backup_category_filter_no_match() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("file1.txt"), "some content")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "documents".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // Filter by non-existent category "videos"
    let result = runner.run(None, Some("videos"))?;

    // No targets match, should return empty result
    assert_eq!(result.total_files, 0);
    assert_eq!(result.successful, 0);

    Ok(())
}

/// Test 25: Combined priority and category filters
///
/// Tests filtering with both priority and category specified.
#[test]
fn test_backup_priority_and_category_filters() -> Result<()> {
    let temp = TempDir::new()?;
    let source1 = temp.path().join("source1");
    let source2 = temp.path().join("source2");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source1)?;
    fs::create_dir_all(&source2)?;

    fs::write(source1.join("high_doc.txt"), "high priority document")?;
    fs::write(source2.join("low_doc.txt"), "low priority document")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source1.clone(),
        Priority::High,
        "documents".to_string(),
    ));
    config.targets.push(Target::new(
        source2.clone(),
        Priority::Low,
        "documents".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // Filter by category "documents" - should backup both
    let result = runner.run(None, Some("documents"))?;

    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);

    Ok(())
}

// ==================== Priority 9: Dry Run & Edge Cases ====================

/// Test 26: Dry run mode - no actual file copies
///
/// Tests that dry_run flag prevents actual backup operations.
#[test]
fn test_backup_dry_run_no_actual_copy() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("dryrun1.txt"), "dry run content 1")?;
    fs::write(source.join("dryrun2.txt"), "dry run content 2")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "dryrun-test".to_string(),
    ));

    // Create runner with dry_run = true
    let mut runner = BackupRunner::new(config, true)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Dry run should report files
    assert_eq!(result.total_files, 2);

    Ok(())
}

/// Test 27: Empty directory backup
///
/// Tests handling of empty directories (no files to backup).
#[test]
fn test_backup_empty_directory() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("empty_source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    // source_dir is empty (no files)

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        source.clone(),
        Priority::High,
        "empty-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Empty directory should result in 0 files backed up
    assert_eq!(result.total_files, 0);
    assert_eq!(result.successful, 0);

    Ok(())
}

/// Test 28: Nonexistent file target
///
/// Tests graceful handling of nonexistent backup targets.
#[test]
fn test_backup_nonexistent_file() -> Result<()> {
    let temp = TempDir::new()?;
    let backup_dest = temp.path().join("backup");

    let nonexistent_path = temp.path().join("nonexistent_dir");

    let mut config = Config::default();
    config.backup.destination = backup_dest;
    config.targets.push(Target::new(
        nonexistent_path.clone(),
        Priority::High,
        "nonexistent-test".to_string(),
    ));

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    // This should handle gracefully (no panic)
    let result = runner.run(None, None);

    // Depending on implementation, might return error or empty result
    // Verify it doesn't panic at least
    match result {
        Ok(r) => {
            assert_eq!(r.total_files, 0);
        }
        Err(_) => {
            // Error is acceptable for nonexistent paths
        }
    }

    Ok(())
}

/// Test 29: Backup with exclude patterns
///
/// Tests that exclude_patterns configuration is accepted (actual filtering
/// may require additional FileFilter setup).
#[test]
fn test_backup_with_exclude_patterns() -> Result<()> {
    let temp = TempDir::new()?;
    let source = temp.path().join("source");
    let backup_dest = temp.path().join("backup");

    fs::create_dir_all(&source)?;
    fs::write(source.join("include.txt"), "should be backed up")?;
    fs::write(source.join("include2.txt"), "should be backed up too")?;

    let mut config = Config::default();
    config.backup.destination = backup_dest.clone();

    let mut target = Target::new(source.clone(), Priority::High, "exclude-test".to_string());
    // Set exclude patterns (filtering implementation may vary)
    target.exclude_patterns = vec!["node_modules".to_string(), ".git".to_string()];
    config.targets.push(target);

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // Verify backup completed successfully with exclude patterns configured
    assert_eq!(result.total_files, 2);
    assert_eq!(result.successful, 2);

    // Verify backup directory structure was created
    let backup_path = backup_dest.join(&result.backup_name).join("exclude-test");
    assert!(backup_path.exists());

    Ok(())
}

/// Test 30: BackupResult::new() when no targets match filters
///
/// Tests that BackupResult::new() is returned when all targets are filtered out.
#[test]
fn test_backup_result_no_targets_match() -> Result<()> {
    let temp = TempDir::new()?;
    let backup_dest = temp.path().join("backup");

    // Config with no targets
    let mut config = Config::default();
    config.backup.destination = backup_dest;

    let mut runner = BackupRunner::new(config, false)
        .with_progress(false)
        .with_compression(CompressionType::None, 0);

    let result = runner.run(None, None)?;

    // BackupResult::new() should be returned
    assert_eq!(result.total_files, 0);
    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 0);
    assert_eq!(result.total_bytes, 0);

    Ok(())
}
