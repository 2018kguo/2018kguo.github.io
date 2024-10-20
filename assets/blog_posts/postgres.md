## Loose collection of notes from [Postgres Internals](https://www.interdb.jp/pg/index.html) 

### Chapter 2

Individual processes use the work_mem memory area for performing join and sort operations, pages for tables and indexes are loaded into a shared buffer pool

### Chapter 3 

SQL statement lifecycle: parse -> analyze to query tree -> rewrite query tree -> generate plan -> execute plan 

parsenodes.h defines the tree structure for the query tree

costsize.c defines functions that the planner uses for query optimization, i.e cost_seqscan()

selectivity of query predicates is used as a factor during cost estimation and estimated w/ info from pg_stats. either calculated by referencing the most common values for a column or histogram_bounds for ints/floats 

SQL statement lifecycle: parse -> analyze to query tree -> rewrite query tree -> generate plan -> execute plan 

Something like `Sort Method: external sort  Disk: 10000kB` in the explain output means that a temporary file was created for joining/sorting. Temporary files are created in base/pg_tmp

Hash join in Postgres uses a 2 phase in memory hash join if the size of the inner table is 1/4 or less of work_mem, otherwise the hybrid hash join is used w/ the skew method

If # of tables in a query is smaller than around 12, dynamic programming can be applied to get the optimal plan. It uses a genetic algorithm for many tables

In the dynamic programming approach, the cheapest join path of each combination of tables is calculated and reused at the next level. i.e with 3 tables you calculate cheapest join paths for {A, B}, {A, C}, {B, C} in level 2 so in level 3 you only need to calculate the join path of the unused table + the combination of 2 tables from the previous step and compare 3 results.