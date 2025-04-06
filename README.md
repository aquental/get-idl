# get-idl

On the Solana blockchain, download the IDL given an address.

```rust
    let program_address = "ADcaide4vBtKuyZQqdU689YqEGZMCmS4tL35bdTv9wJa";
    let cluster = Cluster::Devnet;

    generate_local_idl(program_address, cluster)?;
```

Will generate a local file (_ADcaide4vBtKuyZQqdU689YqEGZMCmS4tL35bdTv9wJa.json_) if successful.
