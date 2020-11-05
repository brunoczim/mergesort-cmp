# mergesort-cmp
Mergesort versions comparison

# Sample Output
```
$ cargo run --release

Using seed 2181328093307058093

Case set tiny, min size = 1, max size = 50, cases = 5120
Target sequential took 0.011646894s
Target parallel took 0.380864456s

Case set small, min size = 100, max size = 500, cases = 1280
Target sequential took 0.046946026s
Target parallel took 0.124656579s

Case set medium, min size = 1000, max size = 5000, cases = 320
Target sequential took 0.13429407s
Target parallel took 0.093843813s

Case set big, min size = 10000, max size = 50000, cases = 80
Target sequential took 0.429879759s
Target parallel took 0.218245473s

Case set large, min size = 100000, max size = 500000, cases = 20
Target sequential took 1.174278587s
Target parallel took 0.524987829s

Case set huge, min size = 1000000, max size = 5000000, cases = 5
Target sequential took 3.696640408s
Target parallel took 1.6022886079999998s
```
