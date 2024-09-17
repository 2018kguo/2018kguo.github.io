### Loose collection of notes from https://www.interdb.jp/pg/

### Chapter 2

Individual processes use the work_mem memory area for performing join and sort operations, pages for tables and indexes are loaded into a shared buffer pool

### Chapter 3 

SQL statement lifecycle: parse -> analyze to query tree -> rewrite query tree -> generate plan -> execute plan 

parsenodes.h defines the tree structure for the query tree

costsize.c defines functions that the planner uses for query optimization, i.e cost_seqscan()

selectivity of query predicates is used as a factor during cost estimation and estimated w/ info from pg_stats. either calculated by referencing the most common values for a column or histogram_bounds for ints/floats 

SQL statement lifecycle: parse -> analyze to query tree -> rewrite query tree -> generate plan -> execute plan 

parsenodes.h defines the tree structure for the query tree

costsize.c defines functions that the planner uses for query optimization, i.e cost_seqscan()

selectivity of query predicates is used as a factor during cost estimation and estimated w/ info from pg_stats. either calculated by referencing the most common values for a column or histogram_bounds for ints/floats 
SQL statement lifecycle: parse -> analyze to query tree -> rewrite query tree -> generate plan -> execute plan 
