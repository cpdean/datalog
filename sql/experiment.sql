/* ullman says i can do datalog with sql. lets see */

/* run a sqlite interpreter
sqlite3 test.db
*/

create table links (
    link_key integer primary key ,
    source text,
    target text
);

-- link(a, b).
-- link(b, c).

-- link(x, y).
-- link(y, z).
-- link(z, g).

insert into links (source, target) values ('a', 'b');
insert into links (source, target) values ('b', 'c');

insert into links (source, target) values ('x', 'y');
insert into links (source, target) values ('y', 'z');
insert into links (source, target) values ('z', 'g');

select
*
from
links;

-- path(X, Y) :- link(X, Y).
-- path(X, Y) :- link(X, Z), path(Z, Y).

/* relational algebra

P(X, Y) = L(X, Y) /union /pi{x, y}(P(X, Z) /innerjoin P(Z, Y))

*/

with recursive P as (
    select
    source,
    target
    from links
    --union all
    union
    select
    a.source,
    b.target
    from links a
    join P b on
    a.target = b.source
)
select
*
from
P;
