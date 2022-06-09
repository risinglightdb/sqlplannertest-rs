-- sql1
CREATE TABLE t1(v1 int);

/*

*/

-- sql2
CREATE TABLE t2(v2 int);

/*

*/

-- Test whether join works correctly.
SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;

/*
=== logical
I'm a naive db, so I don't now how to process
CREATE TABLE t1(v1 int);

CREATE TABLE t2(v2 int);

CREATE TABLE t3(v3 int);
SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;


=== physical
I'm a naive db, so I don't now how to process
CREATE TABLE t1(v1 int);

CREATE TABLE t2(v2 int);

CREATE TABLE t3(v3 int);
SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;
*/

