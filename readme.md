# Bairiak
**Bit-wise flag management for efficient handling of multiple boolean states.**

`Bairiak` is a simple and efficient solution for managing numerous boolean flags using bitwise operations. By condensing multiple flags into a single integer, it improves performance and reduces memory overhead.

### Showcase

#### Without Bairiak
Using individual boolean flags for various states:
```json
{
    "isReceiverVerified": true,
    "isSupplierVerified": true,
    "isInvoiceDataVerified": false,
    "areProductsVerified": false,
    "isAccounted": false,
    "isAlreadyPaid": true,
    "isPaid": false,
    "isAmountApproved": false,
    "isCounterpartyApproved": false,
    "isExpenseTypeApproved": false,
    "isAmountApprovalNeeded": false,
    "isCounterpartyApprovalNeeded": false,
    "isExpenseTypeApprovalNeeded": false,
    "canReturnToValidated": false,
    "canReject": false,
    "isReimbursement": false
}
```
#### With Bairiak
```json
{
    "bairiak": 35
}
```

### Example
1. **Define a YAML spec of all the flags you need.** 

    The enum and variant names must follow CamelCase conventions. An enum with no variants is considered invalid. The type of the `Bairiak` number will range from `u8` to `u128`, depending on the number of variants. The maximum number of flags you can define is 128.
    ```yaml
    # bairiak_spec.yaml
    enums:
    - name: DocumentFlags
        variants:
        - IsReceiverVerified
        - IsSupplierVerified
        - IsInvoiceDataVerified
        - AreProductsVerified
        - IsAccounted
        - IsAlreadyPaid
        - IsPaid
        - IsAmountApproved
        - IsCounterpartyApproved
        - IsExpenseTypeApproved
        - IsAmountApprovalNeeded
        - IsCounterpartyApprovalNeeded
        - IsExpenseTypeApprovalNeeded
        - CanReject
        - CanReturnToValidated
        - IsReimbursement
    ```
2.	**Add the generation function to your `build.rs` file.**
    
    This will automatically generate the corresponding Rust code for your enums.
    ```rust
    // build.rs
    use bairiak::{generate_bairiak_enums, BairiakError};

    fn main() -> Result<(), BairiakError> {
        generate_bairiak_enums("bairiak_spec.yaml", "src/bairiak_enums.rs")?;
        Ok(())
    }
    ```
3.	**Use `generate_bairiak()` to create a `Bairiak` instance.**

    This function takes a HashSet of flags that are set to true. Each flag corresponds to a specific variant in your enum.
    ```rust 
    let mut flags = HashSet::new();
    flags.insert(DocumentFlags::IsReceiverVerified);
    flags.insert(DocumentFlags::IsSupplierVerified);
    flags.insert(DocumentFlags::IsAlreadyPaid);

    let bairiak = generate_bairiak(flags);
    ```
4.	**Use the `is_true` or `is_false` methods to check flag states.**

    These methods allow you to check if a given flag is set (true) or unset (false) for a specific `Bairiak` value.
    ```rust
    println!("{:?}", bairiak.is_false(DocumentFlags::IsPaid));
    println!("{:?}", bairiak.is_true(DocumentFlags::IsAlreadyPaid));
    ```

### Features

- **Bitwise operations for efficiency**: Instead of handling individual booleans, you manage all flags as bits within a single integer.
- **Supports up to 128 flags per enum**: Depending on the number of variants, `Bairiak` automatically chooses the smallest integer type (`u8` to `u128`).
- **Flexible YAML configuration**: Define your flags in a simple and human-readable format.
- **Compile-time generation**: The flags are automatically generated in Rust code at compile time through `build.rs`, reducing boilerplate.

### Performance Benefits

By using bitwise operations, `Bairiak` minimizes memory usage and improves performance when dealing with large sets of boolean flags. Instead of checking multiple individual flags, you manipulate a single integer, making the code more efficient and easier to maintain.

### Limitations

- **Up to 128 flags per enum**: This limit is due to the use of `u128` as the largest integer type for bitwise operations.
- **Valid Enum Names**: Enum and variant names must follow CamelCase conventions to ensure compatibility with the generated Rust code.

### Installation

To add `Bairiak` to your project, include it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
bairiak = "0.1.0"  # Replace with the actual version you're using
```

### Contributing

We welcome contributions! Please feel free to submit issues, fork the repository, and make pull requests.