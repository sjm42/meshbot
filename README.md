# Meshbot

Very basic example usage:

```
cargo run -- -v -a 10.x.y.z:4403
```

* use `-d` flag for more verbose logging
* use `-b` to respond to broadcast packets
* use `-h` for help/usage.

---

# extra tricks

Cross-build for Asus RT-AX59U with arm64 cpu running OpenWrt

```
cross build --release --target aarch64-unknown-linux-musl
```

Cross-build for Asus RT-AX53U with mips cpu running OpenWrt

```
cross +nightly build -Z build-std --release --target mipsel-unknown-linux-musl
```

