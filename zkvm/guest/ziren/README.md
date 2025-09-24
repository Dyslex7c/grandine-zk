## zkMIPS

Load ZKM env vars to current shell
```
source ~/.zkm-toolchain/env
```

Command for successful build
```
$HOME/.cargo/bin/cargo +nightly-2025-06-30 build --release --target mipsel-zkm-zkvm-elf
```

We have to explicitly mention cargo PATH here as it gets overriden by ZKM toolchain
```
$ which cargo
/Users/username/.zkm-toolchain/rust-toolchain-aarch64-apple-darwin-20250717/bin/cargo
```

Navigate to the root directory, and run
```
export ZKM_ELF_zkvm_guest_ziren=$(pwd)/target/mipsel-zkm-zkvm-elf/release/zkvm_guest_ziren
```