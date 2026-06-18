#!/usr/bin/env python3
"""Convert a CHIRP-exported CSV into the ftx1-mm CSV format.

Tested with exports from FT-897 / FT-897D images, but the input format is
CHIRP's generic CSV — any radio CHIRP supports should work.

Usage:
    python3 chirp_ft897_to_ftx1.py chirp_export.csv > ftx1_channels.csv

Limitations:
- D-STAR (DV) channels are skipped — not supported by FTX-1.
- Wide FM (WFM) is skipped — not a standard FTX-1 channel mode.
- Cross-tone setups (CHIRP "Cross" mode) and reverse-tone (TSQL-R, DTCS-R)
  are skipped — they don't map cleanly to FTX-1's single Squelch Type field.
  Configure those manually after import.
- CHIRP "CW" has no upper/lower distinction; mapped to CW-U. Edit if needed.
- Channel tags longer than 12 chars are truncated.
- Skipped rows are reported on stderr with the line number and reason.
"""
import csv
import sys

FTX1_HEADER = [
    "Channel Number", "Frequency (Hz)", "Memory Tag", "Mode", "Channel Type",
    "Squelch Type", "Shift (Hz)", "Clarifier Offset (Hz)",
    "Rx Clarifier Enabled", "Tx Clarifier Enabled",
    "CTCSS Tone", "DCS Tone",
]

MODE_MAP = {
    "FM":  "FM",
    "NFM": "FM-N",
    "AM":  "AM",
    "LSB": "LSB",
    "USB": "USB",
    "CW":  "CW-U",
    "DIG": "DATA-U",
    "PKT": "DATA-FM",
}

DUPLEX_MAP = {
    "":  "Simplex",
    "+": "PlusShift",
    "-": "MinusShift",
}

SQL_MAP = {
    "":     "CtcssOff",
    "Tone": "CtcssEnc",
    "TSQL": "CtcssEncDec",
    "DTCS": "Dcs",
}


def convert(in_path: str) -> None:
    with open(in_path, newline="") as f:
        reader = csv.DictReader(f)
        writer = csv.writer(sys.stdout)
        writer.writerow(FTX1_HEADER)

        for i, row in enumerate(reader, start=2):
            def skip(reason: str) -> None:
                print(f"# skipped line {i}: {reason}", file=sys.stderr)

            try:
                freq_mhz = float(row.get("Frequency") or "0")
            except ValueError:
                skip(f"bad frequency '{row.get('Frequency')}'")
                continue
            if freq_mhz <= 0:
                skip("empty frequency")
                continue
            freq_hz = int(round(freq_mhz * 1_000_000))

            try:
                ch_num = f"{int(row.get('Location') or '0'):05d}"
            except ValueError:
                skip(f"bad Location '{row.get('Location')}'")
                continue

            mode_chirp = (row.get("Mode") or "").strip()
            mode = MODE_MAP.get(mode_chirp)
            if mode is None:
                skip(f"unsupported mode '{mode_chirp}'")
                continue

            duplex = (row.get("Duplex") or "").strip()
            shift = DUPLEX_MAP.get(duplex)
            if shift is None:
                skip(f"unsupported duplex '{duplex}' (split not handled)")
                continue

            tone_kind = (row.get("Tone") or "").strip()
            sql = SQL_MAP.get(tone_kind)
            if sql is None:
                skip(f"unsupported tone mode '{tone_kind}'")
                continue

            tag = (row.get("Name") or "").strip()[:12].ljust(12)

            ctcss = (row.get("rToneFreq") or "100.0").strip() or "100.0"
            dcs_raw = (row.get("DtcsCode") or "023").strip() or "023"
            try:
                dcs = str(int(dcs_raw))
            except ValueError:
                dcs = dcs_raw

            writer.writerow([
                ch_num, freq_hz, tag, mode, "MemoryChannel", sql, shift,
                0, "RxClarifierOff", "TxClarifierOff", ctcss, dcs,
            ])


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(f"Usage: {sys.argv[0]} <chirp_export.csv> > ftx1_channels.csv")
    convert(sys.argv[1])


if __name__ == "__main__":
    main()
