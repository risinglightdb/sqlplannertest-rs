-----
CREATE TABLE t1(v1 int);

hello, world!

-----
CREATE TABLE t2(v2 int);

hello, world!

-----
SELECT * FROM t1, t2 WHERE t1.v1 = t2.v2;

hello, world!

