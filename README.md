# mergesort-cmp
Mergesort versions comparison

# Sample Output
```
$ cargo run --release

Using seed 5292963374513801910

Case set tiny, min size = 1, max size = 50, cases = 5120
Target sequential took 0.011347687s
Target parallel logical took 0.375873679s
Target parallel physical took 0.168607758s
Target parallel 2x logical took 0.647049685s
Target parallel 4x logical took 1.354904455s

Case set small, min size = 100, max size = 500, cases = 1280
Target sequential took 0.043768787s
Target parallel logical took 0.127853191s
Target parallel physical took 0.072415277s
Target parallel 2x logical took 0.209176453s
Target parallel 4x logical took 0.39332226s

Case set medium, min size = 1000, max size = 5000, cases = 320
Target sequential took 0.138396526s
Target parallel logical took 0.098632664s
Target parallel physical took 0.093434939s
Target parallel 2x logical took 0.123353921s
Target parallel 4x logical took 0.166651703s

Case set big, min size = 10000, max size = 50000, cases = 80
Target sequential took 0.371339556s
Target parallel logical took 0.195796908s
Target parallel physical took 0.231383048s
Target parallel 2x logical took 0.214892021s
Target parallel 4x logical took 0.225878362s

Case set large, min size = 100000, max size = 500000, cases = 20
Target sequential took 1.034503713s
Target parallel logical took 0.48346664s
Target parallel physical took 0.605199719s
Target parallel 2x logical took 0.510837767s
Target parallel 4x logical took 0.501835585s

Case set huge, min size = 1000000, max size = 5000000, cases = 5
Target sequential took 3.678073097s
Target parallel logical took 1.644359321s
Target parallel physical took 2.019907155s
Target parallel 2x logical took 1.7007762579999999s
Target parallel 4x logical took 1.663855624s
```
