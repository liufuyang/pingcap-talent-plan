# pingcap-talent-plan

[![Build Status](https://travis-ci.org/liufuyang/pingcap-talent-plan.svg?branch=master)](https://travis-ci.org/liufuyang/pingcap-talent-plan)

An awesome Rust course at
https://github.com/pingcap/talent-plan/tree/master/rust

Project 3 bench result (`kvs` is my implementation, `kvs-pingcap` is default tutorial implementation):
```
set_bench/kvs           time:   [14.407 ms 14.825 ms 15.261 ms]
                        change: [-7.3120% -4.0526% -0.9356%] (p = 0.02 < 0.05)
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe
set_bench/kvs-pingcap   time:   [13.051 ms 13.134 ms 13.243 ms]
                        change: [-7.0440% -3.3348% +0.0598%] (p = 0.08 > 0.05)
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe

get_bench/kvs/8         time:   [2.2198 us 2.2426 us 2.2694 us]
                        change: [-3.7726% -1.0535% +1.5769%] (p = 0.47 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
get_bench/kvs/12        time:   [2.5493 us 2.5857 us 2.6269 us]
                        change: [-2.5729% -0.6849% +1.2644%] (p = 0.50 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
get_bench/kvs/16        time:   [3.6672 us 3.7875 us 3.9301 us]
                        change: [-15.665% -10.417% -4.4655%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
get_bench/kvs-pingcap/8 time:   [2.5248 us 2.5502 us 2.5779 us]
                        change: [-2.3361% -0.1403% +2.1802%] (p = 0.90 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
get_bench/kvs-pingcap/12
                        time:   [2.8776 us 2.9150 us 2.9549 us]
                        change: [-4.0438% -1.6899% +0.5829%] (p = 0.16 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
get_bench/kvs-pingcap/16
                        time:   [3.8280 us 3.9414 us 4.0657 us]
                        change: [+3.3198% +7.6554% +12.463%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Gnuplot not found, disabling plotting
```