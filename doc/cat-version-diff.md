# FTX-1 CAT Operation Reference Manual — Version Diff

## Sources

| Version | Link |
| :--- | :--- |
| 2507-B | (local PDF) |
| 2508-C | <https://www.yaesu.com/Files/4CB893D7-1018-01AF-FA97E9E9AD48B50C/FTX-1_CAT_OM_ENG_2508-C.pdf> |

## Primary Differences

### MC (Memory Channel) Read Command

The only significant technical change is the MC Read command format:

| Detail | 2507-B | 2508-C |
| :--- | :--- | :--- |
| Read format | `MC;` | `MCp1;` |
| Parameter | none | **p1:** `0` = MAIN, `1` = SUB |

**Impact:** Software must now specify the side when reading the active memory channel.
This matches the pattern of other side-specific commands (e.g. AG, BC).

### Code change

`CmdMc::read()` in `src/ftx1.rs` was updated to require a `Side` argument:

```rust
// Before (2507-B)
pub fn read(&self) -> Vec<u8> { ... }          // sends MC;

// After (2508-C)
pub fn read(&self, side: Side) -> Vec<u8> { ... }  // sends MC0; or MC1;
```

## Unchanged

- Firmware requirement: MAIN Firmware Ver. 1.08 or later
- Hardware setup (USB CAT-1/CAT-2, UART CAT-3)
- Full alphabetical command list (AB through ZI)
- Table 3 (MENU Chart) and Table 4 (MY SYMBOL Chart)
