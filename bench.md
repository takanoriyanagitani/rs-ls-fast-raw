# Simple benchmarks

## MacOS

CPU: M3 Max

| tool           | user | sys  | cpu | total | rate   | ratio |
|:--------------:|:----:|:----:|:---:|:-----:|:------:|:-----:|
| rs-ls-fast-raw | 0.01 | 0.05 | 97% | 0.069 | 1,320K | 1000% |
| find           | 0.01 | 0.17 | 98% | 0.181 |   500K |  380% |
| ls             | 0.55 | 0.16 | 99% | 0.702 |   130K |  100% |

## Linux

CPU: Core i7-13700 

| tool           | user | sys  | cpu | total | rate   | ratio |
|:--------------:|:----:|:----:|:---:|:-----:|:------:|:-----:|
| rs-ls-fast-raw | 0.00 | 0.02 | 98% | 0.020 | 4,567K |  145% |
| find           | 0.00 | 0.02 | 99% | 0.028 | 3,262K |  104% |
| ls             | 0.02 | 0.01 | 99% | 0.029 | 3,150K |  100% |
