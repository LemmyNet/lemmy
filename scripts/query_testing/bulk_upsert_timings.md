# post_read -> post_actions bulk upsert timings

## normal, 1 month: 491s

Insert on post_actions (cost=0.57..371215.69 rows=0 width=0) (actual time=169235.026..169235.026 rows=0 loops=1) Conflict Resolution: UPDATE Conflict Arbiter Indexes: post_actions_pkey Tuples Inserted: 5175253 Conflicting Tuples: 0 -> Index Scan using idx_post_read_published_desc on post_read (cost=0.57..371215.69 rows=5190811 width=58) (actual time=47.762..39310.551 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) Planning Time: 0.234 ms Trigger for constraint post_actions_person_id_fkey: time=118828.666 calls=5175253 Trigger for constraint post_actions_post_id_fkey: time=203098.355 calls=5175253 JIT: Functions: 6 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.448 ms, Inlining 0.000 ms, Optimization 0.201 ms, Emission 44.721 ms, Total 45.369 ms Execution Time: 491991.365 ms (15 rows)

## disabled triggers, keep pkey, on conflict: 167s

Insert on post_actions (cost=0.57..371215.69 rows=0 width=0) (actual time=167261.176..167261.176 rows=0 loops=1) Conflict Resolution: UPDATE Conflict Arbiter Indexes: post_actions_pkey Tuples Inserted: 5175253 Conflicting Tuples: 0 -> Index Scan using idx_tmp_1 on post_read (cost=0.57..371215.69 rows=5190811 width=58) (actual time=5.604..59193.030 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) Planning Time: 0.147 ms JIT: Functions: 6 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.490 ms, Inlining 0.000 ms, Optimization 0.197 ms, Emission 3.989 ms, Total 4.675 ms Execution Time: 167261.807 ms

## disabled triggers, with pkey, insert only: 91s

Insert on post_actions (cost=0.57..371215.69 rows=0 width=0) (actual time=91820.768..91820.769 rows=0 loops=1) -> Index Scan using idx_tmp_1 on post_read (cost=0.57..371215.69 rows=5190811 width=58) (actual time=5.482..40066.185 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) Planning Time: 0.098 ms JIT: Functions: 5 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.490 ms, Inlining 0.000 ms, Optimization 0.208 ms, Emission 3.894 ms, Total 4.592 ms Execution Time: 91821.724 ms

## disabled triggers, no pkey, insert only: 57s

Insert on post_actions (cost=0.57..371215.69 rows=0 width=0) (actual time=56797.431..56797.432 rows=0 loops=1) -> Index Scan using idx_tmp_1 on post_read (cost=0.57..371215.69 rows=5190811 width=58) (actual time=4.827..27903.829 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) Planning Time: 0.096 ms JIT: Functions: 5 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.390 ms, Inlining 0.000 ms, Optimization 0.232 ms, Emission 3.373 ms, Total 3.994 ms Execution Time: 56798.022 ms

## disabled triggers, merge instead of upsert: 77s

Merge on post_actions pa (cost=34.06..280379.97 rows=0 width=0) (actual time=76988.823..76988.825 rows=0 loops=1) Tuples: inserted=5175253 -> Hash Left Join (cost=34.06..280379.97 rows=1098137 width=28) (actual time=8.109..12202.884 rows=5175253 loops=1) Hash Cond: ((post_read.person_id = pa.person_id) AND (post_read.post_id = pa.post_id)) -> Index Scan using idx_tmp_1 on post_read (cost=0.56..274581.25 rows=1098137 width=22) (actual time=8.094..11432.132 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) -> Hash (cost=19.40..19.40 rows=940 width=14) (actual time=0.003..0.004 rows=0 loops=1) Buckets: 1024 Batches: 1 Memory Usage: 8kB -> Seq Scan on post_actions pa (cost=0.00..19.40 rows=940 width=14) (actual time=0.003..0.003 rows=0 loops=1) Planning Time: 0.468 ms JIT: Functions: 17 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.897 ms, Inlining 0.000 ms, Optimization 0.399 ms, Emission 7.650 ms, Total 8.946 ms Execution Time: 76989.946 ms

## disabled triggers, merge, no pkey: 39s

Merge on post_actions pa (cost=297488.30..303957.64 rows=0 width=0) (actual time=39009.474..39009.477 rows=0 loops=1) Tuples: inserted=5175253 -> Hash Right Join (cost=297488.30..303957.64 rows=1098137 width=28) (actual time=3412.832..5353.677 rows=5175253 loops=1) Hash Cond: ((pa.person_id = post_read.person_id) AND (pa.post_id = post_read.post_id)) -> Seq Scan on post_actions pa (cost=0.00..19.40 rows=940 width=14) (actual time=0.004..0.005 rows=0 loops=1) -> Hash (cost=274581.25..274581.25 rows=1098137 width=22) (actual time=3412.178..3412.180 rows=5175253 loops=1) Buckets: 131072 (originally 131072) Batches: 64 (originally 16) Memory Usage: 7169kB -> Index Scan using idx_tmp_1 on post_read (cost=0.56..274581.25 rows=1098137 width=22) (actual time=8.495..2299.278 rows=5175253 loops=1) Index Cond: (published > (CURRENT_DATE - '6 mons'::interval)) Planning Time: 0.465 ms JIT: Functions: 17 Options: Inlining false, Optimization false, Expressions true, Deforming true Timing: Generation 0.988 ms, Inlining 0.000 ms, Optimization 0.350 ms, Emission 8.127 ms, Total 9.465 ms Execution Time: 39011.515 ms

## same as above, full table: 425s

Merge on post_actions pa (cost=1478580.50..1520165.83 rows=0 width=0) (actual time=425751.243..425751.245 rows=0 loops=1) Tuples: inserted=33519660 -> Hash Right Join (cost=1478580.50..1520165.83 rows=7091220 width=28) (actual time=72968.237..120866.662 rows=33519660 loops=1) Hash Cond: ((pa.person_id = pr.person_id) AND (pa.post_id = pr.post_id)) -> Seq Scan on post_actions pa (cost=0.00..19.40 rows=940 width=14) (actual time=0.004..0.004 rows=0 loops=1) -> Hash (cost=1330661.20..1330661.20 rows=7091220 width=22) (actual time=72967.590..72967.591 rows=33519660 loops=1) Buckets: 131072 (originally 131072) Batches: 256 (originally 64) Memory Usage: 7927kB -> Seq Scan on post_read pr (cost=0.00..1330661.20 rows=7091220 width=22) (actual time=103.545..51892.728 rows=33519660 loops=1) Planning Time: 0.393 ms JIT: Functions: 14 Options: Inlining true, Optimization true, Expressions true, Deforming true Timing: Generation 0.840 ms, Inlining 11.303 ms, Optimization 45.211 ms, Emission 40.003 ms, Total 97.357 ms Execution Time: 425753.438 ms

## disabled triggers, merge, with pkey, full table: 587s

Merge on post_actions pa (cost=19.47..1367909.58 rows=0 width=0) (actual time=587295.757..587295.759 rows=0 loops=1) Tuples: inserted=33519660 -> Hash Left Join (cost=19.47..1367909.58 rows=7091220 width=28) (actual time=77.291..46496.679 rows=33519660 loops=1) Hash Cond: ((pr.person_id = pa.person_id) AND (pr.post_id = pa.post_id)) -> Seq Scan on post_read pr (cost=0.00..1330661.20 rows=7091220 width=22) (actual time=77.266..41178.528 rows=33519660 loops=1) -> Hash (cost=19.40..19.40 rows=5 width=14) (actual time=0.006..0.007 rows=0 loops=1) Buckets: 1024 Batches: 1 Memory Usage: 8kB -> Seq Scan on post_actions pa (cost=0.00..19.40 rows=5 width=14) (actual time=0.006..0.006 rows=0 loops=1) Filter: (read IS NULL) Planning Time: 0.428 ms JIT: Functions: 16 Options: Inlining true, Optimization true, Expressions true, Deforming true Timing: Generation 0.922 ms, Inlining 6.324 ms, Optimization 37.862 ms, Emission 33.076 ms, Total 78.183 ms Execution Time: 587297.207 ms (15 rows)

## disabled triggers, merge, no pkey, full table: 359s

## disabled triggers, merge, no pkey, person_post_aggs after post_read: 1260s

## disabled triggers, no pkey, post_read + person_post_aggs union all with group by insert (no upsert or merge): 402s

### Merge example:

```sql
EXPLAIN ANALYZE MERGE INTO post_actions pa
USING post_read pr ON (pa.person_id = pr.person_id
    AND pa.post_id = pr.post_id
)
WHEN MATCHED THEN
    UPDATE SET
        read = pr.published
WHEN NOT MATCHED THEN
    INSERT (person_id, post_id, read)
        VALUES (pr.person_id, pr.post_id, pr.published);
```

## comment aggregate bulk update: 3881s / 65m
