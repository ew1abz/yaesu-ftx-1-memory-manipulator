# Memory Channel Fields

Status of all fields that can be associated with a memory channel.

| Field | Status | Source | Notes |
| :--- | :---: | :--- | :--- |
| Memory Channel Number | ✅ | `CMD_MR` | |
| Receive Frequency | ✅ | `CMD_MR` | |
| Offset Direction | ✅ | `CMD_MR` | Simplex / Plus Shift / Minus Shift |
| Offset Frequency | ❌ | — | The actual repeater offset in Hz; not in `CMD_MR` response |
| Operating Mode | ✅ | `CMD_MR` | 15 modes supported; **C4FM-DN and C4FM-VW missing** from `Mode` enum |
| Clarifier Offset (Hz) | ✅ | `CMD_MR` | Signed value; sign encodes direction |
| Clarifier Direction | ✅ | `CMD_MR` | Derived from sign of clarifier offset |
| RX Clarifier Status | ✅ | `CMD_MR` | ON / OFF |
| TX Clarifier Status | ✅ | `CMD_MR` | ON / OFF |
| SQL Type / Tone Mode | ✅ | `CMD_MR` | OFF / CTCSS ENC-DEC / CTCSS ENC / DCS / PR FREQ / REV TONE |
| CTCSS Tone Frequency | ✅ | `CMD_CN` | 50 standard tones |
| DCS Code | ✅ | `CMD_CN` | 104 codes |
| Memory Channel Tag | ✅ | `CMD_MT` | Up to 12 ASCII characters |
| ARS (Auto Repeater Shift) | ❌ | — | |
| Split Status | ❌ | — | |
| IPO (Pre-Amp status) | ❌ | — | |
| DNF (Digital Notch Filter) | ❌ | — | |
| DNR (Digital Noise Reduction level) | ❌ | — | |
| Narrow Filter Status | ❌ | — | |
| RF Attenuator Status | ❌ | — | |
| Noise Blanker Level | ❌ | — | |
| AGC Function | ❌ | — | FAST / MID / SLOW / AUTO |

## Notes

**Power:** Per-channel power levels are not programmable via CAT. Global max power
limits can be set, but operating power must be adjusted manually on the radio.

**`CMD_MR` response layout** (`MRcccccfffffffffooooortmqdd s;`, 27 param bytes):

```
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
