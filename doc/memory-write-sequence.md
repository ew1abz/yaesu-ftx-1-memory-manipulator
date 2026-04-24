# Memory Channel Write Sequence

How `--write-radio` commits a full memory channel (including CTCSS/DCS tones)
to the FTX-1, and why the obvious approach with `MW` alone does not work.

## TL;DR

The `MW` command cannot store tones. A single call to `MW` silently resets the
channel's tone to the default (code 12 / 100.0 Hz). The working approach is to
build up the full main-side VFO state with individual CAT commands and then
commit it to memory with `AM`.

Per-channel sequence (see [`write_radio_data`](../src/main.rs) for the source):

| # | Command                           | Purpose                                                       |
| - | --------------------------------- | ------------------------------------------------------------- |
| 1 | `VM P1=0 P2=11`                   | Main side → Memory mode                                       |
| 2 | `MC P1=0 Pn=ccccc`                | Select target channel on main                                 |
| 3 | `VM P1=0 P2=00`                   | Main side → VFO mode (anchors AM to the selected slot)        |
| 4 | `MD P1=0 P2=4`                    | Force FM first, because OS only applies in FM mode            |
| 5 | `OS P1=0 P2=<shift>`              | Set repeater shift (simplex / plus / minus / ARS)             |
| 6 | `MD P1=0 P2=<mode>`               | Now set the real operating mode                               |
| 7 | `FA nnnnnnnnn`                    | Set main VFO-A frequency                                      |
| 8 | `CT P1=0 P2=<sql>`                | Set SQL type (see P2 encoding note below)                     |
| 9 | `CN P1=0 P2=0 Pn=ccc`             | Set CTCSS tone on main                                         |
| 10 | `CN P1=0 P2=1 Pn=ccc`            | Set DCS tone on main                                           |
| 11 | `AM`                              | Commit full main VFO state to the currently selected channel  |
| 12 | `MT ccccc<tag>`                   | Write the 12-char channel tag                                  |

`MW` is not used during write. It is still used elsewhere to probe whether the
legacy path behaves as documented (it does not).

## Gotchas

### 1. `MW` does not persist tones

Setting P8 (SQL type) in `MW` to any non-zero value without updating the tone
register first is not sufficient — the stored tone code stays at whatever was
in memory before (or is reset to 12 / 100.0 Hz on a fresh write).

Priming `CN` *before* `MW` has no effect. `MW` ignores the radio's current
CN setting. The only way to persist a tone is through `AM` (main-side
VFO-to-memory transfer), which copies the full VFO state — including the
currently-set tone — into the selected memory slot.

### 2. `AM` targets the *currently selected* main memory channel

`AM` takes no channel argument. It writes to whichever channel is currently
active on main. This means `MC` must be sent before `AM`, and the VM → MC → VM
dance is required so that:

- `MC` records the target slot (and the channel's existing settings
  are loaded as a side-effect — though we overwrite most of them).
- The final `VM P2=00` puts main into VFO mode so the subsequent state-setting
  commands (`MD`, `OS`, `FA`, `CT`, `CN`) modify the VFO rather than the memory
  slot.

### 3. `OS` is only accepted in FM mode

The CAT spec notes: *"This command can be activated only with an FM mode."*
In practice the radio silently ignores `OS` when the current mode is not FM,
and any previous shift value (from the prior iteration of the write loop)
persists into the saved channel.

Mitigation: always force FM via `MD` first, then send `OS`, then switch to the
real target mode. For non-FM channels this correctly clears any stale shift
state back to Simplex.

### 4. `CT` P2 encoding differs from `MW` P8

The two commands use the *same* numeric field, with codes 1 and 2 swapped:

| Code | `MW` P8                  | `CT` P2                      |
| :--: | :----------------------- | :--------------------------- |
| 0    | CTCSS OFF                | CTCSS OFF                    |
| 1    | CTCSS ENC/DEC (both on)  | CTCSS ENC on / DEC off       |
| 2    | CTCSS ENC (encode only)  | CTCSS ENC on / DEC on        |
| 3    | DCS                      | DCS                          |
| 4    | PR FREQ                  | PR FREQ                      |
| 5    | REV TONE                 | REV TONE                     |

`CmdCt::set` in [`src/ftx1.rs`](../src/ftx1.rs) handles the swap so callers can
pass the same `SqlType` enum used elsewhere.

### 5. `VM` parameter format

`VM P1 P2;` where:

- **P1** — side: `0` = MAIN, `1` = SUB.
- **P2** — two-char mode code: `00` = VFO, `10` = MT, `11` = Memory,
  `20` = PMS, `21` = P-01L..P-50U, `51` = 5 MHz Band Memory, `91` = EMG.

So "main-side to VFO mode" is `VM000;` (one + two = three digits of params).

## CAT commands added for this flow

All defined in [`src/ftx1.rs`](../src/ftx1.rs):

- `CmdVm` — VFO / Memory mode switch.
- `CmdFa` — set main VFO-A frequency.
- `CmdAm` — save main VFO to the selected memory channel.
- `CmdMd` — set operating mode on a given side.
- `CmdOs` — set repeater shift on a given side.
- `CmdCt` — set SQL type on a given side (with P2 swap).

## Validation

Round-trip test on 16 memory channels (mix of FM / FM-N / AM / USB / CW-U /
LSB modes, with and without CTCSS tones, simplex and minus-shift repeaters):
`write-radio` from backup CSV → `read-radio` to new CSV → `diff` → byte
identical.

## Prior approaches that did not work

For reference, none of these persist the tone:

| Attempt                                              | Result                                    |
| :--------------------------------------------------- | :---------------------------------------- |
| `MW` alone with P8 = CTCSS ENC                       | Tone reset to code 12 on the channel      |
| `CN` (any side) → `MW`                               | Tone not stored                           |
| `MW` → `CN` (main) → `MT`                            | Works on first channel only, not repeatable |
| `MC` (sub) → `CN` (sub) → `MW`                       | Tone not stored                           |
| `MC` (sub) → `CN` (sub) → `BM`                       | `BM` saves the actual sub VFO (stuck on 144 MHz), corrupts the channel |
| `MC` (sub) → `MB` → `CN` → `BM`                      | `MB` loads into a display buffer, not the real sub VFO-B; `BM` still writes 144 MHz |
| `AM` after `MW`                                      | `AM` overwrites mode/shift/sql with VFO defaults |
| `MW` after `AM`                                      | `MW` wipes the tone just committed by `AM` |

The P9 field in `MW` is strictly `'00'` — any other value triggers a `?;`
rejection from the radio.
