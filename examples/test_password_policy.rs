// Test password policy behavior
use backup_suite::crypto::PasswordPolicy;

fn main() {
    let policy = PasswordPolicy::default();

    println!("=== Testing Password Policy ===\n");

    // Test 1: Very weak password
    println!("Test 1: Password = \"weak\"");
    println!("{}", policy.display_report("weak"));

    // Test 2: Short password
    println!("\n{}", "=".repeat(50));
    println!("Test 2: Password = \"abc123\"");
    println!("{}", policy.display_report("abc123"));

    // Test 3: Common password
    println!("\n{}", "=".repeat(50));
    println!("Test 3: Password = \"password\"");
    println!("{}", policy.display_report("password"));

    // Test 4: Sequential pattern
    println!("\n{}", "=".repeat(50));
    println!("Test 4: Password = \"12345678\"");
    println!("{}", policy.display_report("12345678"));

    // Test 5: Medium password
    println!("\n{}", "=".repeat(50));
    println!("Test 5: Password = \"MyBackup2024\"");
    println!("{}", policy.display_report("MyBackup2024"));

    // Test 6: Strong password
    println!("\n{}", "=".repeat(50));
    println!("Test 6: Password = \"MyS3cur3!B@ckup#2024\"");
    println!("{}", policy.display_report("MyS3cur3!B@ckup#2024"));

    // Test 7: Check current minimum length
    println!("\n{}", "=".repeat(50));
    println!("Current Policy Settings:");
    println!("  Minimum Length: {}", policy.min_length);
    println!("  Entropy Check: {}", policy.check_entropy);
}
