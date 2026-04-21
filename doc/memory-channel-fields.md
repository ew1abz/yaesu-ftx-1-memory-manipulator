# Memory Channel Fields

Status of all fields that can be associated with a memory channel.

| Field | Status | Source | Notes |
| :--- | :---: | :--- | :--- |
| Memory Channel Number | ✅ | `CMD_MR` | |
| Receive Frequency | ✅ | `CMD_MR` | |
| Offset Direction | ✅ | `CMD_MR` | Simplex / Plus Shift / Minus Shift |
| Offset Frequency | ❌ | — | The actual repeater offset in Hz; not in `CMD_MR` response |
| Operating Mode | ✅ | `CMD_MR` | 17 modes supported (CAT chars `1`–`9`, `A`–`F`, `H`, `I`) |
| Clarifier Offset (Hz) | ✅ | `CMD_MR` | Signed value; sign encodes direction |
| Clarifier Direction | ✅ | `CMD_MR` | Derived from sign of clarifier offset |
| RX Clarifier Status | ✅ | `CMD_MR` | ON / OFF |
| TX Clarifier Status | ✅ | `CMD_MR` | ON / OFF |
| SQL Type / Tone Mode | ✅ | `CMD_MR` | OFF / CTCSS ENC-DEC / CTCSS ENC / DCS / PR FREQ / REV TONE |
| CTCSS Tone Frequency | ✅ | `CMD_CN` | 50 standard tones |
| DCS Code | ✅ | `CMD_CN` | 104 codes |
| Memory Channel Tag | ✅ | `CMD_MT` | Up to 12 ASCII characters |
| Memory Group (M-GRP) | ❌ | — | Per-channel boolean; marks a channel as part of the user-defined M-GRP recall group. Band groups (M-HF, 50MHz, M-AIR, M-VHF, M-UHF) are automatic from frequency. Absent from CAT spec (`CMD_MR`/`CMD_MW`); likely in uncharted bytes `[26..27]`, or via an undocumented CAT command — a USB trace of RT-Systems would clarify |
| ARS (Auto Repeater Shift) | ❌ | — | Per-band; part of `OS` P2=3 and EX/MENU RPT SHIFT |
| Split TX Frequency | ❌ | `MZ` | Per-channel via `MZ` command; not returned by `CMD_MR` |
| IPO / Pre-Amp | ❌ | — | Per-band group (HF/50 MHz, VHF, UHF) via `PA`; not per channel |
| DNF (Auto Notch) | ❌ | — | Per-side (MAIN/SUB) via `BC`; not stored per channel |
| DNR (Noise Reduction level) | ❌ | — | Per-side (MAIN/SUB) via `RL`; not stored per channel |
| Narrow Filter | ❌ | — | Per-side (MAIN/SUB) via `NA`; not stored per channel |
| RF Attenuator | ❌ | — | Radio-global via `RA`; not stored per channel |
| Noise Blanker Level | ❌ | — | Per-side (MAIN/SUB) via `NL`; not stored per channel |
| AGC Function | ❌ | — | Per-side (MAIN/SUB) via `GT`; FAST / MID / SLOW / AUTO |

## Notes

**Power:** Per-channel power levels are not programmable via CAT. Global max power
limits can be set, but operating power must be adjusted manually on the radio.

**`CMD_MR` response layout** (`MRcccccfffffffffooooortmqdd s;`, 27 param bytes):

```text
[2..7]   ccccc  channel (5)
[7..16]  fffffffff  frequency (9)
[16..21] ooooo  clarifier offset with sign (5)
[21]     r  RX clarifier
[22]     t  TX clarifier
[23]     m  mode
[24]     q  channel type
[25]     s  SQL type
[26..27] dd  currently treated as dummy — may encode ARS, split, or other fields
[28]     s  shift direction
```

The two "dummy" bytes at positions 26–27 are uncharted and may carry some of
the unread fields above. Capturing a live radio trace and comparing against the
CAT manual would clarify their meaning.

**Offset Frequency and deriving TX frequency:**

`CMD_MR` encodes the shift *direction* (Simplex / Plus / Minus) per channel, but
the actual offset Hz is **not stored per channel**. It is a global per-band menu
setting accessible via the `EX` (MENU) command:

| Band    | Range      | Resolution   |
| :------ | :--------- | :----------- |
| 28 MHz  | 0–1000 kHz | 10 kHz steps |
| 50 MHz  | 0–4000 kHz | 10 kHz steps |
| 144 MHz | 0–100 MHz  | 50 kHz steps |
| 430 MHz | 0–100 MHz  | 50 kHz steps |

To derive the TX frequency you need both pieces:

```text
tx_freq = rx_freq + offset_hz   (Plus Shift)
tx_freq = rx_freq - offset_hz   (Minus Shift)
tx_freq = rx_freq               (Simplex)
```

where `offset_hz` comes from the `EX` read for the band that channel belongs to.

The EX value is a **snapshot of the radio's current menu state**, not a value
frozen at channel-store time. If the band offset was changed after a channel was
programmed, the derived TX frequency will be incorrect. For this reason, offset
Hz is treated as out-of-scope for the per-channel CSV format.
