# US series: notes

All lengths are millimeters converted from the inch values of the sources (× 25.4, 2 decimals).
Fractions in the catalogs (to 1/32 or 1/64 in) were transcribed by hand from the PDFs. The text
layer of the Charlotte PDFs runs whole numbers into fractions ("115/32" = 1-15/32), so each value was
checked against a rendered page image or against H − G ≈ socket. Pipe `di` is `de − 2e` (average OD
minus twice the minimum wall) unless the source publishes an ID (copper, PEX).

## Files and row counts

| File | Pipes | Fittings |
|---|---|---|
| US-PVC-SCH40.json | 10 (1/2" to 6") | 91 |
| US-PVC-SCH80.json | 10 (1/2" to 6") | 84 |
| US-PVC-DWV.json | 6 (1-1/4" to 6") | 57 |
| US-CU-L.json | 9 (1/2" to 4") | 97 |
| US-CU-M.json | 9 (1/2" to 4") | 97 (same fittings as Type L) |
| US-PEX.json | 7 (3/8" to 2") | 12 |
| US-CPVC-CTS.json | 6 (1/2" to 2") | 52 |

## Sources by series

- PVC Sch 40, Sch 80, CPVC CTS: Charlotte Pipe DC-PR pressure dimensional catalog (one
  manufacturer). Its G (laying length, center to socket bottom), J (45° laying length) and N
  (socket bottom to socket bottom) are used directly as `z`.
- PVC DWV: Charlotte Pipe DC-DWV catalog. The letters were read from the drawings: 1/4, 1/8 and 1/16
  bends A = center to hub bottom; sanitary tee A = center to hub bottom (branch and lower run), B = run
  hub bottom to hub bottom; wye A = run hub bottom to hub bottom, B = intersection to lower hub bottom,
  C = intersection to branch hub bottom; coupling A = gap, B = overall; increaser-reducer N = hub bottom
  to hub bottom.
- Copper L/M: tube sizes and minimum cup depths from the CDA Copper Tube Handbook; laying lengths
  (center to cup shoulder, dims C/D/F/G/A) from the NIBCO wrot catalog C-CF-1225. Fittings are
  the same for Type L and Type M (B16.22 fittings fit both).
- PEX: tubing from the Viega PureFlow submittal (nominal ID published); fittings per the Sioux Chief
  F1807 submittal.

## Choices and open questions

- **Socket depth in PVC/CPVC elbows and tees** is `H − G` (published center-to-face minus laying
  length), not the standard minimum. Charlotte's Sch 40 sockets are 0.12 to 0.5 in deeper than the
  D2466 minimum (for example 1-1/4" elbow: 1.25 in vs 0.938 in; 6": 3.5 in vs 3.0 in); Sch 80 and
  CPVC match their standard minimum within 1/32 in. Where `H − G` was below the minimum or far
  above it, the standard minimum is used and the row says so: Sch 40 reducing tees 3"×1/2" and
  3"×1-1/4" (branch, 1/16 in short, probably rounding), Sch 80 reducing tee 4"×3" (branch H1 − G1 =
  1.56 in vs 1.875 in), CPVC reducing tee 1"×1/2" (branch H1 = 1-29/32 in looks like a misprint; its
  G1 = 17/32 in is kept).
- Sch 80 1-1/4" minimum socket length is 1.290 in in the Charlotte table (I expected 1.250 in from
  memory of D2467); the published value is used.
- **Couplings**: socket = (L − N)/2 from the overall length (PVC, CPVC, DWV); copper couplings use
  the CDA minimum cup depth.
- **Reducers**: socket × socket reducing couplings where the catalog sells one; otherwise flush
  reducer bushings (spigot × socket), with `z = N` = spigot end to the bottom of the small socket (that
  is where a large pipe end would sit in the host socket), `socket2` = standard minimum socket of the
  small size, and `socket` = spigot length SPG (Sch 40) or null (Sch 80, CPVC: not published). Each
  bushing row has a `derived` note. CPVC bushing 2"×1" has N = 7/16 in, which does not agree with its
  L (L − N = 1.375 in, the 1" socket is 0.9 in); kept as published, suspect.
- **Sch 40 3/4" tee vs elbow**: same G (9/16 in) but H differs (1-1/2 vs 1-5/16 in), so their socket
  depths differ. Sch 40 6" elbow H − G = 3.5 in; kept.
- **DWV asymmetric runs**: the format has one `z` per run. For sanitary tees `z` is the lower
  (outlet) end; for wyes `z` is B (intersection to the lower hub). The other run end (B − A for
  sanitary tees, A − B for wyes) is given in the row's `derived` text in mm. The format may need a
  `z3` (or `z_run2`) for these. Sanitary tees are recorded as kind `tee` as asked. DWV 1/16 bend has
  no 1-1/4" size in the catalog. The 4"×4"×1-1/2" wye (B = 0) is assembled from two parts.
- DWV socket depths are the catalog's PVC hub depth C (equal to the D2665 minimums as far as I know);
  coupling sockets from (B − A)/2 agree with them except 1-1/4" to 2" (0.72 to 0.76 in vs 0.687/0.750).
- Only reducing tees and wyes with both run ends of one size are recorded; bullhead tees (branch
  larger) and tees with a reduced run are left out. Reducing elbows are left out (no format field).
- **Copper**: NIBCO's 2-1/2" to 4" tees have a branch laying length longer than the run (e.g. 4":
  2-13/32 run, 2-17/32 branch); reducing tees with small branches can be far shorter on the run than
  the equal tee (4"×4"×2": 1-11/32). Kept as published. The 4" cup depth (2.16 in) comes from the CDA
  table; NIBCO's own cup depth chart stops at 3-1/2". Only one manufacturer's laying lengths were
  used; other makers (Mueller, Elkhart) may differ by 1/16 to 1/8 in.
- **PEX**: no public center-to-end dimensions were found for F1807 crimp fittings (`z` null). `socket`
  is the F1807 minimum fitting end length 0.625 in (marked derived). Fitting rows only for 3/8" to 1"
  (the sizes the cited F1807 submittal covers); 1-1/4" to 2" crimp fittings exist on the market but no
  dimensional source was found, so none are listed. Reducing tees/couplings are sold but left out
  for lack of any dimension. PEX `di` is Viega's nominal ID, which is smaller than `de − 2e` (it allows
  for the wall tolerance).
- CPVC pipe: minimum walls are those of ASTM D2846 as listed by Charlotte (1/2" 0.068 in, not 0.625/11).
  The 1-1/4"×1" CPVC reducing coupling is sold assembled.
- Size labels follow the US market (`1/2"`, `1-1/4"` …); PEX 3/8" is included as asked.

## Fixes made when the files were first validated

- US-PEX: `di` (Viega's nominal ID) was 0.25 to 0.61 mm smaller than `de − 2e` with the F876 minimum
  walls, past the catalog check's 0.15 mm. `di` was kept, since sizing reads it, and `e` was set to
  (de − di) / 2 rounded to 0.01 mm (3/8" 1.91, 1/2" 1.91, 3/4" 2.59, 1" 3.34, 1-1/4" 4.08,
  1-1/2" 4.84, 2" 6.30), a wall that includes the tolerance.
