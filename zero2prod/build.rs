fn main() {
    // Recompile if any files in the migration directory changes.
    // include additions and deletions.
    println!("cargo::rerun-if-changed=migrations/");
}
